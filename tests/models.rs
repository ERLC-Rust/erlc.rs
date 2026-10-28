//! Deserialization of response models.

use erlc::models::ServerInfo;
use serde_json::json;

const V2_SERVER: &str = include_str!("fixtures/v2_server.json");

#[test]
fn deserializes_server_info_fixture() {
    let info: ServerInfo = serde_json::from_str(V2_SERVER).unwrap();
    assert_eq!(info.name, "API Test");
    assert_eq!(info.owner_id, 1);
    assert_eq!(info.co_owner_ids, vec![2, 3]);
    assert_eq!(info.current_players, 12);
    assert_eq!(info.max_players, 40);
    assert_eq!(info.join_key, "APIServer");
    assert_eq!(info.acc_verified_req, "Disabled");
    assert!(info.team_balance);
}

#[test]
fn ignores_unknown_fields() {
    let mut value: serde_json::Value = serde_json::from_str(V2_SERVER).unwrap();
    value["SomeFutureField"] = json!({"nested": [1, 2, 3]});
    value["Players"] = json!([]);
    let info: ServerInfo = serde_json::from_value(value).unwrap();
    assert_eq!(info.name, "API Test");
}

#[test]
fn rejects_missing_required_fields() {
    let mut value: serde_json::Value = serde_json::from_str(V2_SERVER).unwrap();
    value.as_object_mut().unwrap().remove("Name");
    let err = serde_json::from_value::<ServerInfo>(value).unwrap_err();
    assert!(err.to_string().contains("Name"), "{err}");
}

#[test]
fn serializes_with_original_field_names() {
    let info: ServerInfo = serde_json::from_str(V2_SERVER).unwrap();
    let round_trip = serde_json::to_value(&info).unwrap();
    let original: serde_json::Value = serde_json::from_str(V2_SERVER).unwrap();
    assert_eq!(round_trip, original);
}

#[test]
fn deserializes_bans() {
    let bans: std::collections::BTreeMap<u64, String> =
        serde_json::from_str(include_str!("fixtures/v1_bans.json")).unwrap();
    assert_eq!(bans.get(&100).map(String::as_str), Some("PlayerName"));
    assert_eq!(bans.len(), 2);
}

/// Redacted capture from a live server with no players online.
mod live_capture {
    use std::collections::BTreeMap;

    use erlc::models::ServerSnapshot;

    #[test]
    fn empty_requested_fields_are_empty_arrays() {
        let s: ServerSnapshot =
            serde_json::from_str(include_str!("fixtures/live_v2_server_empty.json")).unwrap();
        assert_eq!(s.players, Some(vec![]));
        assert_eq!(s.queue, Some(vec![]));
        assert_eq!(s.vehicles, Some(vec![]));
        assert_eq!(s.join_logs.unwrap().len(), 4);
        assert_eq!(s.command_logs.unwrap()[0].player.user_id(), Some(1003));
        let staff = s.staff.unwrap();
        assert_eq!(
            (staff.admins.len(), staff.mods.len(), staff.helpers.len()),
            (8, 1, 2)
        );
    }

    #[test]
    fn bans_use_numeric_ids() {
        let bans: BTreeMap<u64, String> =
            serde_json::from_str(include_str!("fixtures/live_v1_bans.json")).unwrap();
        assert_eq!(bans.len(), 4);
        assert_eq!(bans.get(&5000).map(String::as_str), Some("Banned1"));
    }
}

mod snapshot {
    use erlc::models::{Permission, Player, ServerSnapshot, Staff, Timestamp};

    const FULL: &str = include_str!("fixtures/v2_server_full.json");

    fn full() -> ServerSnapshot {
        serde_json::from_str(FULL).unwrap()
    }

    #[test]
    fn deserializes_every_optional_field() {
        let s = full();
        assert_eq!(s.info.name, "API Test");
        assert_eq!(s.players.as_ref().unwrap().len(), 2);
        assert!(s.staff.is_some());
        assert_eq!(s.join_logs.as_ref().unwrap().len(), 2);
        assert_eq!(s.queue, Some(vec![102, 103]));
        assert_eq!(s.kill_logs.as_ref().unwrap().len(), 1);
        assert_eq!(s.command_logs.as_ref().unwrap().len(), 1);
        assert_eq!(s.mod_calls.as_ref().unwrap().len(), 2);
        assert_eq!(s.emergency_calls.as_ref().unwrap().len(), 1);
        assert_eq!(s.vehicles.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn players() {
        let players = full().players.unwrap();
        let sheriff = &players[0];
        assert_eq!(sheriff.player.name(), "PlayerName");
        assert_eq!(sheriff.player.user_id(), Some(100));
        assert_eq!(sheriff.team, "Sheriff");
        assert_eq!(sheriff.callsign.as_deref(), Some("5D-550"));
        assert_eq!(sheriff.permission, Permission::ServerModerator);
        let location = sheriff.location.as_ref().unwrap();
        assert_eq!(location.location_x, 1084.965);
        assert_eq!(location.street_name, "Park Street");

        let civilian = &players[1];
        assert_eq!(civilian.callsign, None);
        assert_eq!(civilian.permission, Permission::Normal);
        assert_eq!(civilian.wanted_stars, 3);
        assert_eq!(civilian.location.as_ref().unwrap().building_number, "");
    }

    #[test]
    fn player_optional_fields_may_be_missing() {
        let player: Player = serde_json::from_str(
            r#"{"Player":"A:1","Team":"Civilian","Permission":"Server Helper"}"#,
        )
        .unwrap();
        assert_eq!(player.callsign, None);
        assert_eq!(player.location, None);
        assert_eq!(player.wanted_stars, 0);
        assert_eq!(
            player.permission,
            Permission::Other("Server Helper".to_owned())
        );
        assert_eq!(player.permission.as_str(), "Server Helper");
    }

    #[test]
    fn staff_maps_use_numeric_ids() {
        let staff = full().staff.unwrap();
        assert_eq!(
            staff.admins.get(&77573259).map(String::as_str),
            Some("AdminOne")
        );
        assert_eq!(staff.mods.len(), 2);
        assert_eq!(
            staff.helpers.get(&168691872).map(String::as_str),
            Some("HelperOne")
        );
    }

    #[test]
    fn staff_groups_default_to_empty() {
        let staff: Staff = serde_json::from_str(r#"{"Admins":{"1":"A"}}"#).unwrap();
        assert_eq!(staff.admins.len(), 1);
        assert!(staff.mods.is_empty() && staff.helpers.is_empty());
    }

    #[test]
    fn logs() {
        let s = full();
        let join = &s.join_logs.as_ref().unwrap()[1];
        assert!(!join.join);
        assert_eq!(join.timestamp, Timestamp::from_secs(1704614500));

        let kill = &s.kill_logs.as_ref().unwrap()[0];
        assert_eq!(kill.killer.name(), "PlayerName");
        assert_eq!(kill.killed.name(), "OtherPlayer");

        assert_eq!(s.command_logs.as_ref().unwrap()[0].command, ":h");

        let calls = s.mod_calls.unwrap();
        assert_eq!(calls[0].moderator.as_ref().unwrap().name(), "PlayerName");
        assert_eq!(calls[1].moderator, None);
    }

    #[test]
    fn emergency_calls_and_vehicles() {
        let s = full();
        let call = &s.emergency_calls.as_ref().unwrap()[0];
        assert_eq!(call.team, "Police");
        assert_eq!(call.caller, 168691872);
        assert_eq!(call.position, [-654.6, 666.5]);
        assert_eq!(call.call_number, 400);
        assert!(call.players.is_empty());

        let vehicles = s.vehicles.unwrap();
        assert_eq!(vehicles[0].texture.as_deref(), Some("Livery Name"));
        assert_eq!(vehicles[1].texture, None);
        assert_eq!(vehicles[0].color_hex, "#ff4444");
    }

    #[test]
    fn unrequested_fields_are_none() {
        let s: ServerSnapshot = serde_json::from_str(super::V2_SERVER).unwrap();
        assert!(s.players.is_none() && s.staff.is_none() && s.vehicles.is_none());
    }

    #[test]
    fn empty_and_null_optional_fields() {
        let mut value: serde_json::Value = serde_json::from_str(super::V2_SERVER).unwrap();
        value["Players"] = serde_json::json!([]);
        value["Vehicles"] = serde_json::Value::Null;
        let s: ServerSnapshot = serde_json::from_value(value).unwrap();
        assert_eq!(s.players, Some(vec![]));
        assert_eq!(s.vehicles, None);
    }

    #[test]
    fn round_trips_through_serde() {
        let s = full();
        let json = serde_json::to_string(&s).unwrap();
        let again: ServerSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(s, again);
    }
}
