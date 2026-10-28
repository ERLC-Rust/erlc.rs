//! Request format: paths, query strings and headers, checked against a mock server.

use erlc::models::Staff;
use erlc::{Client, ServerQuery};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: &str = "test-server-key-0123456789";
const V2_SERVER: &str = include_str!("fixtures/v2_server.json");

fn client(server: &MockServer) -> Client {
    Client::builder()
        .server_key(KEY)
        .base_url(server.uri())
        .build()
        .unwrap()
}

#[tokio::test]
async fn server_info_sends_authenticated_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v2/server"))
        .and(header("server-key", KEY))
        .and(header("accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(V2_SERVER, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let info = client(&server).server_info().await.unwrap();
    assert_eq!(info.name, "API Test");

    let requests = server.received_requests().await.unwrap();
    let request = &requests[0];
    assert_eq!(request.url.query(), None, "no query string without options");
    assert!(request.headers.get("authorization").is_none());
    let user_agent = request.headers.get("user-agent").unwrap().to_str().unwrap();
    assert!(user_agent.starts_with("erlc-rs/"), "{user_agent}");
}

#[tokio::test]
async fn trims_whitespace_around_the_key() {
    let server = MockServer::start().await;
    Mock::given(header("server-key", KEY))
        .respond_with(ResponseTemplate::new(200).set_body_raw(V2_SERVER, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .server_key(format!("  {KEY}\n"))
        .base_url(server.uri())
        .build()
        .unwrap();
    client.server_info().await.unwrap();
}

#[tokio::test]
async fn keeps_base_url_path_prefix() {
    let server = MockServer::start().await;
    Mock::given(path("/proxy/v2/server"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(V2_SERVER, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = Client::builder()
        .server_key(KEY)
        .base_url(format!("{}/proxy/", server.uri()))
        .build()
        .unwrap();
    client.server_info().await.unwrap();
}

const FULL: &str = include_str!("fixtures/v2_server_full.json");

async fn mock_server_json(body: &'static str) -> (MockServer, Client) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
        .mount(&server)
        .await;
    let client = client(&server);
    (server, client)
}

async fn only_query(server: &MockServer) -> Option<String> {
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    requests[0].url.query().map(str::to_owned)
}

#[tokio::test]
async fn server_with_sends_selected_flags() {
    let (server, client) = mock_server_json(FULL).await;
    let snapshot = client
        .server_with(ServerQuery::new().kill_logs().players())
        .await
        .unwrap();
    assert!(snapshot.players.is_some());
    assert_eq!(
        only_query(&server).await.as_deref(),
        Some("Players=true&KillLogs=true")
    );
}

#[tokio::test]
async fn server_with_all_sends_every_flag() {
    let (server, client) = mock_server_json(FULL).await;
    client.server_with(ServerQuery::all()).await.unwrap();
    assert_eq!(
        only_query(&server).await.as_deref(),
        Some(
            "Players=true&Staff=true&JoinLogs=true&Queue=true&KillLogs=true\
             &CommandLogs=true&ModCalls=true&EmergencyCalls=true&Vehicles=true"
        )
    );
}

macro_rules! shortcut_test {
    ($name:ident, $flag:literal, $len:expr) => {
        #[tokio::test]
        async fn $name() {
            let (server, client) = mock_server_json(FULL).await;
            let items = client.$name().await.unwrap();
            assert_eq!(items.len(), $len);
            assert_eq!(
                only_query(&server).await.as_deref(),
                Some(concat!($flag, "=true"))
            );
        }
    };
}

shortcut_test!(players, "Players", 2);
shortcut_test!(join_logs, "JoinLogs", 2);
shortcut_test!(queue, "Queue", 2);
shortcut_test!(kill_logs, "KillLogs", 1);
shortcut_test!(command_logs, "CommandLogs", 1);
shortcut_test!(mod_calls, "ModCalls", 2);
shortcut_test!(emergency_calls, "EmergencyCalls", 1);
shortcut_test!(vehicles, "Vehicles", 2);

#[tokio::test]
async fn staff_shortcut() {
    let (server, client) = mock_server_json(FULL).await;
    let staff = client.staff().await.unwrap();
    assert_eq!(staff.mods.len(), 2);
    assert_eq!(only_query(&server).await.as_deref(), Some("Staff=true"));
}

#[tokio::test]
async fn shortcuts_return_empty_when_field_is_omitted() {
    let (_server, client) = mock_server_json(V2_SERVER).await;
    assert!(client.players().await.unwrap().is_empty());
    assert!(client.vehicles().await.unwrap().is_empty());
    assert_eq!(client.staff().await.unwrap(), Staff::default());
}

#[tokio::test]
async fn bans_uses_v1_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/server/bans"))
        .and(header("server-key", KEY))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(include_str!("fixtures/v1_bans.json"), "application/json"),
        )
        .expect(1)
        .mount(&server)
        .await;
    let bans = client(&server).bans().await.unwrap();
    assert_eq!(bans.get(&101).map(String::as_str), Some("OtherPlayer"));
}
