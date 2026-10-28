# Testing your code

You should not need a real server key, or a live server, to test code that uses `erlc`.

## Mock the API

Point the client at a local mock server with `base_url`. This example uses
[wiremock](https://crates.io/crates/wiremock):

```toml
[dev-dependencies]
wiremock = "0.6"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde_json = "1"
```

```rust,no_run
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn counts_players() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v2/server"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Name": "Test", "OwnerId": 1, "CoOwnerIds": [], "CurrentPlayers": 1,
            "MaxPlayers": 40, "JoinKey": "test", "AccVerifiedReq": "Disabled",
            "TeamBalance": false,
            "Players": [{"Player": "Someone:1", "Team": "Police", "Permission": "Normal"}]
        })))
        .mount(&server)
        .await;

    let client = erlc::Client::builder()
        .server_key("test-key")
        .base_url(server.uri())
        .build()
        .unwrap();

    assert_eq!(client.players().await.unwrap().len(), 1);
}
```

Mock error responses the same way, for example a 429 with a `Retry-After` header or a
422 for an offline server, to test your error handling.

## Build models from JSON

Models are `#[non_exhaustive]`, so create test data by deserializing JSON rather than with
struct literals:

```rust
use erlc::models::Player;

let player: Player = serde_json::from_str(
    r#"{"Player": "Someone:1", "Team": "Civilian", "Permission": "Normal"}"#,
).unwrap();
assert_eq!(player.player.user_id(), Some(1));
```

## Live checks

Keep live tests out of your normal test run, and only send read requests from them:

```rust,no_run
#[tokio::test]
#[ignore = "needs ERLC_SERVER_KEY and calls the live API"]
async fn live_status() {
    let client = erlc::Client::new(std::env::var("ERLC_SERVER_KEY").unwrap()).unwrap();
    client.server_info().await.unwrap();
}
```

Run them explicitly with `cargo test -- --ignored`. Never put a real key in CI logs or
source code.
