//! Client-side rate-limit handling, shared state between servers, and client options.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use erlc::{Client, Error};
use serde_json::json;
use wiremock::matchers::{any, header, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: &str = "test-server-key-0123456789";
const OTHER_KEY: &str = "other-server-key-9876543210";
const GLOBAL_KEY: &str = "global-api-key-abcdef";
const V2_SERVER: &str = include_str!("fixtures/v2_server.json");

fn client(server: &MockServer) -> Client {
    Client::builder()
        .server_key(KEY)
        .base_url(server.uri())
        .build()
        .unwrap()
}

fn ok() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(V2_SERVER, "application/json")
}

fn limited(bucket: &str, retry_after: &str) -> ResponseTemplate {
    ResponseTemplate::new(429)
        .insert_header("X-RateLimit-Bucket", bucket)
        .insert_header("Retry-After", retry_after)
        .set_body_json(json!({"code": 4001, "message": "You are being rate limited!"}))
}

fn local_limit(err: &Error) -> &erlc::RateLimited {
    match err {
        Error::RateLimited(rl) if rl.is_local() => rl,
        other => panic!("expected a local rate limit, got {other:?}"),
    }
}

#[tokio::test]
async fn requests_after_a_429_are_not_sent() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(limited("global", "30"))
        .expect(1)
        .mount(&server)
        .await;
    let client = client(&server);

    let first = client.server_info().await.unwrap_err();
    assert!(matches!(&first, Error::RateLimited(rl) if !rl.is_local()));

    for _ in 0..3 {
        let err = client.players().await.unwrap_err();
        let rl = local_limit(&err);
        assert_eq!(rl.bucket(), Some("global"));
        assert!(rl.retry_after().unwrap() <= Duration::from_secs(30));
        assert!(err.to_string().ends_with("(request not sent)"));
    }
}

#[tokio::test]
async fn requests_resume_after_retry_after() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(limited("global", "0.3"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(any()).respond_with(ok()).mount(&server).await;
    let client = client(&server);

    assert!(client.server_info().await.is_err());
    assert!(matches!(
        client.server_info().await,
        Err(Error::RateLimited(_))
    ));
    tokio::time::sleep(Duration::from_millis(400)).await;
    client.server_info().await.unwrap();
}

#[tokio::test]
async fn exhausted_bucket_is_not_sent_until_reset() {
    let reset = SystemTime::now() + Duration::from_secs(30);
    let reset = reset
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ok().insert_header("X-RateLimit-Bucket", "global")
                .insert_header("X-RateLimit-Limit", "35")
                .insert_header("X-RateLimit-Remaining", "0")
                .insert_header("X-RateLimit-Reset", reset.as_str()),
        )
        .expect(1)
        .mount(&server)
        .await;
    let client = client(&server);

    client.server_info().await.unwrap();
    let info = client.rate_limit("global").unwrap();
    assert_eq!(info.limit, Some(35));
    assert_eq!(info.remaining, Some(0));

    let err = client.server_info().await.unwrap_err();
    assert_eq!(local_limit(&err).bucket(), Some("global"));
}

#[tokio::test]
async fn command_cooldown_does_not_block_reads() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(limited("command", "30"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .respond_with(ok())
        .expect(1)
        .mount(&server)
        .await;
    let client = client(&server);

    assert!(client.run_command(":h one").await.is_err());
    let err = client.run_command(":h two").await.unwrap_err();
    assert_eq!(local_limit(&err).bucket(), Some("command"));
    client.server_info().await.unwrap();
}

#[tokio::test]
async fn global_cooldown_blocks_commands() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(limited("global", "30"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let client = client(&server);

    assert!(client.server_info().await.is_err());
    let err = client.run_command(":h hi").await.unwrap_err();
    assert_eq!(local_limit(&err).bucket(), Some("global"));
}

#[tokio::test]
async fn with_server_key_sends_the_new_key_and_shares_settings() {
    let server = MockServer::start().await;
    Mock::given(header("server-key", OTHER_KEY))
        .and(header("authorization", GLOBAL_KEY))
        .respond_with(ok())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(header("server-key", KEY))
        .and(header("authorization", GLOBAL_KEY))
        .respond_with(ok())
        .expect(1)
        .mount(&server)
        .await;

    let first = Client::builder()
        .server_key(KEY)
        .global_key(GLOBAL_KEY)
        .base_url(server.uri())
        .build()
        .unwrap();
    let second = first.with_server_key(OTHER_KEY).unwrap();
    second.server_info().await.unwrap();
    first.server_info().await.unwrap();

    assert!(matches!(
        first.with_server_key(" "),
        Err(Error::InvalidInput(_))
    ));
}

#[tokio::test]
async fn command_cooldown_is_per_server() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(header("server-key", KEY))
        .respond_with(limited("command", "30"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(header("server-key", OTHER_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"message": "Success"})))
        .expect(1)
        .mount(&server)
        .await;
    let first = client(&server);
    let second = first.with_server_key(OTHER_KEY).unwrap();

    assert!(first.run_command(":h hi").await.is_err());
    second.run_command(":h hi").await.unwrap();
}

#[tokio::test]
async fn global_cooldown_is_shared_between_servers() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(limited("global", "30"))
        .expect(1)
        .mount(&server)
        .await;
    let first = client(&server);
    let second = first.with_server_key(OTHER_KEY).unwrap();

    assert!(first.server_info().await.is_err());
    let err = second.server_info().await.unwrap_err();
    assert_eq!(local_limit(&err).bucket(), Some("global"));
}

#[tokio::test]
async fn custom_user_agent_and_http_client() {
    let server = MockServer::start().await;
    Mock::given(any()).respond_with(ok()).mount(&server).await;
    let client = Client::builder()
        .server_key(KEY)
        .base_url(server.uri())
        .user_agent("my-bot/1.0")
        .http_client(erlc::reqwest::Client::new())
        .build()
        .unwrap();

    client.server_info().await.unwrap();
    let requests = server.received_requests().await.unwrap();
    let user_agent = requests[0]
        .headers
        .get("user-agent")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        user_agent.starts_with("my-bot/1.0 erlc-rs/"),
        "{user_agent}"
    );
}

#[test]
fn global_key_is_validated_and_redacted() {
    assert!(matches!(
        Client::builder().server_key(KEY).global_key("").build(),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        Client::builder()
            .server_key(KEY)
            .user_agent("bad\nagent")
            .build(),
        Err(Error::InvalidInput(_))
    ));

    let builder = Client::builder().server_key(KEY).global_key(GLOBAL_KEY);
    let debug = format!("{builder:?}");
    assert!(
        !debug.contains(GLOBAL_KEY) && !debug.contains(KEY),
        "{debug}"
    );
    let client = builder.build().unwrap();
    let debug = format!(
        "{client:?} {:?}",
        client.with_server_key(OTHER_KEY).unwrap()
    );
    assert!(
        !debug.contains(GLOBAL_KEY) && !debug.contains(OTHER_KEY),
        "{debug}"
    );
}
