//! `run_command`: request format, delivery guarantees and errors.

use std::time::Duration;

use erlc::{Client, Error, ErrorCode};
use serde_json::json;
use wiremock::matchers::{any, body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: &str = "test-server-key-0123456789";

fn client(server: &MockServer) -> Client {
    Client::builder()
        .server_key(KEY)
        .base_url(server.uri())
        .build()
        .unwrap()
}

async fn command_error(template: ResponseTemplate) -> Error {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(template)
        .expect(1)
        .mount(&server)
        .await;
    client(&server).run_command(":h hi").await.unwrap_err()
}

#[tokio::test]
async fn posts_command_as_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/server/command"))
        .and(header("server-key", KEY))
        .and(header("content-type", "application/json"))
        .and(body_json(json!({"command": ":h Hey everyone!"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"message": "Success"})))
        .expect(1)
        .mount(&server)
        .await;

    let response = client(&server)
        .run_command("  :h Hey everyone!\n")
        .await
        .unwrap();
    assert_eq!(response.message, "Success");
}

#[tokio::test]
async fn empty_command_is_rejected_without_a_request() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let client = client(&server);
    for command in ["", "   ", "\n"] {
        let err = client.run_command(command).await.unwrap_err();
        assert!(matches!(err, Error::InvalidInput(_)), "{err:?}");
    }
}

#[tokio::test]
async fn rate_limited_command_is_sent_once() {
    let err = command_error(
        ResponseTemplate::new(429)
            .insert_header("Retry-After", "5")
            .insert_header("X-RateLimit-Bucket", "command"),
    )
    .await;
    let Error::RateLimited(rl) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(rl.bucket(), Some("command"));
    assert_eq!(rl.retry_after(), Some(Duration::from_secs(5)));
}

#[tokio::test]
async fn server_error_is_sent_once() {
    let err = command_error(ResponseTemplate::new(500).set_body_json(json!({
        "message": "Error communicating with Roblox",
        "commandId": "ecfd9342-0acd-4485-bf32-e654b4a829fd"
    })))
    .await;
    let Error::Api(api) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(
        api.command_id(),
        Some("ecfd9342-0acd-4485-bf32-e654b4a829fd")
    );
}

#[tokio::test]
async fn timed_out_command_is_sent_once() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(5)))
        .expect(1)
        .mount(&server)
        .await;
    let client = Client::builder()
        .server_key(KEY)
        .base_url(server.uri())
        .timeout(Duration::from_millis(200))
        .build()
        .unwrap();

    let err = client.run_command(":h hi").await.unwrap_err();
    assert!(
        matches!(&err, Error::Transport(t) if t.is_timeout()),
        "{err:?}"
    );
}

#[tokio::test]
async fn offline_server() {
    let err = command_error(ResponseTemplate::new(422).set_body_json(json!({
        "message": "The private server is currently offline.",
        "commandId": "abc"
    })))
    .await;
    let Error::Api(api) = &err else {
        panic!("{err:?}")
    };
    assert!(api.is_server_offline());
    assert_eq!(
        api.message(),
        Some("The private server is currently offline.")
    );
}

#[tokio::test]
async fn restricted_command_code() {
    let err = command_error(ResponseTemplate::new(403).set_body_json(json!({"code": 4002}))).await;
    let Error::Api(api) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(api.code(), Some(ErrorCode::RestrictedCommand));
}
