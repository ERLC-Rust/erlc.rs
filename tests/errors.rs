//! Mapping of failed requests to `erlc::Error`, and key redaction.

use std::time::Duration;

use erlc::{Client, Error, ErrorCode};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: &str = "test-server-key-0123456789";

async fn respond_with(template: ResponseTemplate) -> (MockServer, Client) {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(template)
        .mount(&server)
        .await;
    let client = Client::builder()
        .server_key(KEY)
        .base_url(server.uri())
        .build()
        .unwrap();
    (server, client)
}

async fn server_info_error(template: ResponseTemplate) -> Error {
    let (_server, client) = respond_with(template).await;
    client.server_info().await.unwrap_err()
}

fn assert_redacted(text: &str) {
    assert!(!text.contains(KEY), "key leaked: {text}");
}

#[tokio::test]
async fn bad_request_with_code_and_message() {
    let err = server_info_error(ResponseTemplate::new(400).set_body_raw(
        r#"{"code":2001,"message":"Duplicate query parameter: Players"}"#,
        "application/json",
    ))
    .await;
    let Error::Api(api) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(api.status(), 400);
    assert_eq!(api.code(), Some(ErrorCode::MalformedServerKey));
    assert_eq!(api.message(), Some("Duplicate query parameter: Players"));
    assert_redacted(&err.to_string());
    assert_redacted(&format!("{err:?}"));
}

#[tokio::test]
async fn forbidden_is_an_auth_error() {
    let err = server_info_error(
        ResponseTemplate::new(403).set_body_raw(r#"{"code":2002}"#, "application/json"),
    )
    .await;
    let Error::Api(api) = &err else {
        panic!("{err:?}")
    };
    assert!(api.is_auth());
    assert_eq!(api.code(), Some(ErrorCode::InvalidServerKey));
    assert_redacted(&err.to_string());
    assert_redacted(&format!("{err:?}"));
}

#[tokio::test]
async fn forbidden_with_empty_body() {
    let err = server_info_error(ResponseTemplate::new(403)).await;
    let Error::Api(api) = &err else {
        panic!("{err:?}")
    };
    assert!(api.is_auth());
    assert_eq!(api.code(), None);
    assert_eq!(api.message(), None);
    assert_eq!(err.to_string(), "ER:LC API returned HTTP 403");
}

#[tokio::test]
async fn server_error_with_error_field() {
    let err = server_info_error(ResponseTemplate::new(500).set_body_raw(
        r#"{"error":"Error communicating with Roblox, please try again. [trace_123456]","code":1001}"#,
        "application/json",
    ))
    .await;
    let Error::Api(api) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(api.code(), Some(ErrorCode::RobloxCommunication));
    assert!(
        api.message()
            .unwrap()
            .starts_with("Error communicating with Roblox")
    );
}

#[tokio::test]
async fn unprocessable_with_command_id() {
    let err = server_info_error(ResponseTemplate::new(422).set_body_raw(
        r#"{"message":"The private server is currently offline.","commandId":"ecfd9342-0acd-4485-bf32-e654b4a829fd"}"#,
        "application/json",
    ))
    .await;
    let Error::Api(api) = &err else {
        panic!("{err:?}")
    };
    assert!(api.is_server_offline());
    assert_eq!(
        api.command_id(),
        Some("ecfd9342-0acd-4485-bf32-e654b4a829fd")
    );
}

#[tokio::test]
async fn module_outdated() {
    let err = server_info_error(
        ResponseTemplate::new(500).set_body_raw(r#"{"code":9999}"#, "application/json"),
    )
    .await;
    let Error::Api(api) = &err else {
        panic!("{err:?}")
    };
    assert!(api.is_module_outdated());
}

#[tokio::test]
async fn plain_text_error_body() {
    let err = server_info_error(ResponseTemplate::new(502).set_body_string("Bad Gateway")).await;
    let Error::Api(api) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(api.message(), Some("Bad Gateway"));
    assert_eq!(api.body(), "Bad Gateway");
}

#[tokio::test]
async fn rate_limited_reads_header_and_bucket() {
    let err = server_info_error(
        ResponseTemplate::new(429)
            .insert_header("Retry-After", "5")
            .insert_header("X-RateLimit-Bucket", "global")
            .set_body_raw(
                r#"{"code":4001,"message":"You are being rate limited!","retry_after":4.2}"#,
                "application/json",
            ),
    )
    .await;
    let Error::RateLimited(rl) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(rl.retry_after(), Some(Duration::from_secs(5)));
    assert_eq!(rl.bucket(), Some("global"));
    assert_eq!(rl.message(), Some("You are being rate limited!"));
    assert_eq!(
        err.to_string(),
        "rate limited by the ER:LC API (bucket `global`); retry after 5.0s"
    );
}

#[tokio::test]
async fn rate_limited_falls_back_to_body_retry_after() {
    let err = server_info_error(
        ResponseTemplate::new(429).set_body_raw(r#"{"retry_after":1.5}"#, "application/json"),
    )
    .await;
    let Error::RateLimited(rl) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(rl.retry_after(), Some(Duration::from_millis(1500)));
    assert_eq!(rl.bucket(), None);
}

#[tokio::test]
async fn invalid_json_on_success_is_a_decode_error() {
    let err = server_info_error(
        ResponseTemplate::new(200).set_body_raw(r#"{"Name": "API Test", "#, "application/json"),
    )
    .await;
    let Error::Decode(decode) = &err else {
        panic!("{err:?}")
    };
    assert_eq!(decode.status(), 200);
    assert!(
        err.to_string()
            .starts_with("could not decode ER:LC API response (HTTP 200)")
    );
}

#[tokio::test]
async fn missing_field_on_success_is_a_decode_error() {
    let err = server_info_error(
        ResponseTemplate::new(200).set_body_raw(r#"{"Name":"API Test"}"#, "application/json"),
    )
    .await;
    assert!(matches!(err, Error::Decode(_)), "{err:?}");
}

#[tokio::test]
async fn slow_response_times_out() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(5)))
        .mount(&server)
        .await;
    let client = Client::builder()
        .server_key(KEY)
        .base_url(server.uri())
        .timeout(Duration::from_millis(200))
        .build()
        .unwrap();

    let err = client.server_info().await.unwrap_err();
    let Error::Transport(transport) = &err else {
        panic!("{err:?}")
    };
    assert!(transport.is_timeout());
    assert_eq!(err.to_string(), "request to the ER:LC API timed out");
    assert_redacted(&format!("{err:?}"));
}

#[tokio::test]
async fn refused_connection_is_a_connect_error() {
    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };
    let client = Client::builder()
        .server_key(KEY)
        .base_url(format!("http://127.0.0.1:{port}"))
        .build()
        .unwrap();

    let err = client.server_info().await.unwrap_err();
    let Error::Transport(transport) = &err else {
        panic!("{err:?}")
    };
    assert!(transport.is_connect(), "{err:?}");
    assert_redacted(&format!("{err:?}"));
}

#[test]
fn invalid_configuration_is_rejected_before_any_request() {
    for result in [
        Client::new(""),
        Client::new("   "),
        Client::new("bad\nkey"),
        Client::builder().build(),
        Client::builder()
            .server_key(KEY)
            .base_url("not a url")
            .build(),
        Client::builder()
            .server_key(KEY)
            .timeout(Duration::ZERO)
            .build(),
    ] {
        assert!(matches!(result, Err(Error::InvalidInput(_))), "{result:?}");
    }
}

#[test]
fn client_and_builder_debug_output_is_redacted() {
    let builder = Client::builder().server_key(KEY);
    assert_redacted(&format!("{builder:?}"));
    let client = builder.build().unwrap();
    assert_redacted(&format!("{client:?}"));
}
