//! Live read-only check: `cargo test --test live -- --ignored --nocapture` with `ERLC_SERVER_KEY` set.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use erlc::Client;
use erlc::models::ServerSnapshot;
use serde_json::Value;

const BASE: &str = "https://api.erlc.gg";
const ALL_FIELDS: &str = "Players=true&Staff=true&JoinLogs=true&Queue=true&KillLogs=true\
    &CommandLogs=true&ModCalls=true&EmergencyCalls=true&Vehicles=true";
const ARRAYS: [&str; 8] = [
    "Players",
    "JoinLogs",
    "Queue",
    "KillLogs",
    "CommandLogs",
    "ModCalls",
    "EmergencyCalls",
    "Vehicles",
];

fn raw_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/raw");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(n) if n.is_f64() => "float",
        Value::Number(_) => "int",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Fetches `path` without the SDK, saves the raw body, and prints rate-limit headers.
async fn capture(http: &erlc::reqwest::Client, key: &str, path: &str, file: &str) -> Vec<u8> {
    let response = http
        .get(format!("{BASE}{path}"))
        .header("server-key", key)
        .header("accept", "application/json")
        .send()
        .await
        .unwrap();
    let status = response.status();
    let limits: Vec<String> = response
        .headers()
        .iter()
        .filter(|(name, _)| {
            name.as_str().starts_with("x-ratelimit") || name.as_str() == "retry-after"
        })
        .map(|(name, value)| format!("{name}={}", value.to_str().unwrap_or("?")))
        .collect();
    let body = response.bytes().await.unwrap().to_vec();
    println!(
        "{}: HTTP {status} [{}]",
        path.split('?').next().unwrap(),
        limits.join(", ")
    );
    std::fs::write(raw_dir().join(file), &body).unwrap();
    assert!(
        status.is_success(),
        "HTTP {status}: {}",
        String::from_utf8_lossy(&body)
    );
    body
}

/// Prints the response structure and enum-like values; no names or IDs.
fn report(v2: &Value, bans: &Value) {
    let obj = v2.as_object().unwrap();
    println!("\ntop-level: {:?}", obj.keys().collect::<Vec<_>>());
    println!("AccVerifiedReq: {:?}", obj.get("AccVerifiedReq"));

    for field in ARRAYS.iter().copied().chain(["Staff"]) {
        match obj.get(field) {
            None => println!("{field}: omitted"),
            Some(Value::Array(items)) => {
                let mut shape: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
                for item in items {
                    match item {
                        Value::Object(map) => map.iter().for_each(|(k, v)| {
                            shape.entry(k).or_default().insert(kind(v));
                        }),
                        other => {
                            shape.entry("<item>").or_default().insert(kind(other));
                        }
                    }
                }
                println!("{field}: {} items, fields {shape:?}", items.len());
            }
            Some(Value::Object(map)) => {
                let groups: BTreeMap<_, _> = map
                    .iter()
                    .map(|(k, v)| {
                        let numeric_keys = v
                            .as_object()
                            .is_some_and(|m| m.keys().all(|id| id.parse::<u64>().is_ok()));
                        (
                            k,
                            (kind(v), v.as_object().map_or(0, |m| m.len()), numeric_keys),
                        )
                    })
                    .collect();
                println!("{field}: (kind, count, numeric keys) {groups:?}");
            }
            Some(other) => println!("{field}: {}", kind(other)),
        }
    }

    let distinct = |field: &str| -> BTreeSet<String> {
        obj.get("Players")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|p| p.get(field).map(|v| v.to_string()))
            .collect()
    };
    println!("Permission values: {:?}", distinct("Permission"));
    println!("Team values: {:?}", distinct("Team"));
    println!("WantedStars values: {:?}", distinct("WantedStars"));
    println!(
        "Location kinds: {:?}",
        obj.get("Players")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|p| p.get("Location").map_or("missing", kind))
            .collect::<BTreeSet<_>>()
    );

    match bans {
        Value::Object(map) => println!(
            "Bans: object, {} entries, numeric keys: {}, value kinds {:?}",
            map.len(),
            map.keys().all(|k| k.parse::<u64>().is_ok()),
            map.values().map(kind).collect::<BTreeSet<_>>()
        ),
        other => println!("Bans: {}", kind(other)),
    }
}

#[tokio::test]
#[ignore = "calls the live ER:LC API; needs ERLC_SERVER_KEY"]
async fn live_read_only() {
    let key = std::env::var("ERLC_SERVER_KEY").expect("set ERLC_SERVER_KEY");

    let info = Client::new(key.clone())
        .unwrap()
        .server_info()
        .await
        .unwrap();
    println!(
        "SDK server_info: OK ({} players online)",
        info.current_players
    );

    let http = erlc::reqwest::Client::new();
    let v2 = capture(
        &http,
        &key,
        &format!("/v2/server?{ALL_FIELDS}"),
        "v2_server_all.json",
    )
    .await;
    let bans = capture(&http, &key, "/v1/server/bans", "v1_bans.json").await;

    report(
        &serde_json::from_slice(&v2).unwrap(),
        &serde_json::from_slice(&bans).unwrap(),
    );

    let snapshot = serde_json::from_slice::<ServerSnapshot>(&v2);
    let bans = serde_json::from_slice::<BTreeMap<u64, String>>(&bans);
    println!(
        "\nServerSnapshot decode: {:?}",
        snapshot.as_ref().map(|_| "OK")
    );
    println!("Bans decode: {:?}", bans.as_ref().map(|_| "OK"));
    snapshot.unwrap();
    bans.unwrap();
}
