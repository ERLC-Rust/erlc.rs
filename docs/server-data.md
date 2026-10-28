# Server data

Everything readable comes from `GET /v2/server`, plus bans from `GET /v1/server/bans`.
All models live in `erlc::models`.

## Server status

```rust,no_run
async fn status(client: &erlc::Client) -> erlc::Result<()> {
    let info = client.server_info().await?;
    println!("name: {}", info.name);
    println!("players: {}/{}", info.current_players, info.max_players);
    println!("owner: {}, co-owners: {:?}", info.owner_id, info.co_owner_ids);
    println!("verification: {}, team balance: {}", info.acc_verified_req, info.team_balance);
    Ok(())
}
```

`ServerInfo` fields:

| Field | Type | Notes |
|---|---|---|
| `name` | `String` | Server name |
| `owner_id` | `u64` | Roblox user ID of the owner |
| `co_owner_ids` | `Vec<u64>` | Roblox user IDs of co-owners |
| `current_players` | `u32` | Players in the server now |
| `max_players` | `u32` | Player limit |
| `join_key` | `String` | Code players use to join |
| `acc_verified_req` | `String` | e.g. `"Disabled"`, `"Email"`, `"Phone/ID"` |
| `team_balance` | `bool` | Whether team balance is on |

## Several datasets in one request

`ServerQuery` selects optional data, and `server_with` fetches it in **one request**.
Prefer this over calling several shortcuts when you need more than one dataset: each call
counts against your [rate limit](rate-limits.md).

```rust,no_run
use erlc::ServerQuery;

async fn dashboard(client: &erlc::Client) -> erlc::Result<()> {
    let snapshot = client
        .server_with(ServerQuery::new().players().staff().vehicles())
        .await?;

    println!("{} is online", snapshot.info.name);
    println!("players: {}", snapshot.players.as_deref().unwrap_or_default().len());
    println!("vehicles: {}", snapshot.vehicles.as_deref().unwrap_or_default().len());
    Ok(())
}
```

`ServerQuery::all()` requests every optional field.

### `None` versus empty

Each optional field of `ServerSnapshot` is an `Option`:

- `None`: you did not request it.
- `Some(vec![])`: you requested it and it is empty, for example no players online.

## Shortcuts

Each shortcut sends one request with one field and returns it directly, or an empty value
if the API omits it.

| Method | Query flag | Returns |
|---|---|---|
| `players()` | `Players` | `Vec<Player>` |
| `staff()` | `Staff` | `Staff` |
| `join_logs()` | `JoinLogs` | `Vec<JoinLog>` |
| `queue()` | `Queue` | `Vec<u64>` (Roblox IDs) |
| `kill_logs()` | `KillLogs` | `Vec<KillLog>` |
| `command_logs()` | `CommandLogs` | `Vec<CommandLog>` |
| `mod_calls()` | `ModCalls` | `Vec<ModCall>` |
| `emergency_calls()` | `EmergencyCalls` | `Vec<EmergencyCall>` |
| `vehicles()` | `Vehicles` | `Vec<Vehicle>` |

## Players

```rust,no_run
use erlc::models::Permission;

async fn list_players(client: &erlc::Client) -> erlc::Result<()> {
    for p in client.players().await? {
        let role = match &p.permission {
            Permission::ServerOwner => "owner",
            Permission::ServerAdministrator => "admin",
            Permission::ServerModerator => "mod",
            Permission::Normal => "player",
            other => other.as_str(),
        };
        println!("{} [{}] team={} callsign={:?}", p.player.name(), role, p.team, p.callsign);
    }
    Ok(())
}
```

`Player` fields:

| Field | Type | Notes |
|---|---|---|
| `player` | `PlayerRef` | See below |
| `team` | `String` | e.g. `"Police"`, `"Sheriff"`, `"Civilian"` |
| `callsign` | `Option<String>` | Only for non-civilian teams |
| `permission` | `Permission` | `Normal`, `ServerModerator`, `ServerAdministrator`, `ServerOwner`, or `Other(String)` |
| `wanted_stars` | `u32` | Wanted level |
| `location` | `Option<Location>` | Map position |

### `PlayerRef`

The API identifies players as `"Name:UserId"`. `PlayerRef` keeps that raw value and splits
it for you:

```rust
use erlc::models::PlayerRef;

let p = PlayerRef::new("Builderman:156");
assert_eq!(p.name(), "Builderman");
assert_eq!(p.user_id(), Some(156));
assert_eq!(p.as_str(), "Builderman:156");
```

If the value has no numeric `:UserId` suffix, `name()` returns the whole value and
`user_id()` returns `None`.

### Locations

`Location` has `location_x`, `location_z`, `postal_code`, `street_name` and
`building_number`. Coordinates are relative to the centre of the map:

- `0, 0` is the map centre.
- Positive `location_x` is right, negative is left.
- Positive `location_z` is down, negative is up.

The [official map images](https://apidocs.erlc.gg/official-map-images) are 5355 × 5355 pixels.

## Staff

Staff groups map Roblox user IDs to usernames. Owners and co-owners are on `ServerInfo`.

```rust,no_run
async fn list_staff(client: &erlc::Client) -> erlc::Result<()> {
    let staff = client.staff().await?;
    for (id, name) in &staff.admins {
        println!("admin {name} ({id})");
    }
    println!("{} mods, {} helpers", staff.mods.len(), staff.helpers.len());
    Ok(())
}
```

## Logs

```rust,no_run
use std::time::SystemTime;

async fn recent_activity(client: &erlc::Client) -> erlc::Result<()> {
    for join in client.join_logs().await? {
        let verb = if join.join { "joined" } else { "left" };
        let when: SystemTime = join.timestamp.into();
        println!("{} {verb} at {when:?}", join.player.name());
    }
    for kill in client.kill_logs().await? {
        println!("{} killed {}", kill.killer.name(), kill.killed.name());
    }
    for log in client.command_logs().await? {
        println!("{}: {}", log.player.name(), log.command);
    }
    for call in client.mod_calls().await? {
        match &call.moderator {
            Some(m) => println!("{} was helped by {}", call.caller.name(), m.name()),
            None => println!("{} is waiting for a moderator", call.caller.name()),
        }
    }
    Ok(())
}
```

| Type | Fields |
|---|---|
| `JoinLog` | `join` (`true` = joined), `timestamp`, `player` |
| `KillLog` | `killed`, `timestamp`, `killer` |
| `CommandLog` | `player`, `timestamp`, `command` |
| `ModCall` | `caller`, `moderator` (`None` until answered), `timestamp` |

### Timestamps

`Timestamp` wraps Unix seconds. Use `as_secs()` for the raw number, or
`to_system_time()` / `SystemTime::from(ts)` to convert.

## Emergency calls

| Field | Type | Notes |
|---|---|---|
| `team` | `String` | Team called, e.g. `"Police"` |
| `caller` | `u64` | Roblox ID of the caller |
| `players` | `Vec<u64>` | Roblox IDs of responders |
| `position` | `[f64; 2]` | `[x, z]`, same coordinates as `Location` |
| `started_at` | `Timestamp` | When the call started |
| `call_number` | `u64` | Call number |
| `description` | `String` | Caller's description |
| `position_descriptor` | `String` | Location description |

## Vehicles

| Field | Type | Notes |
|---|---|---|
| `name` | `String` | Vehicle model |
| `owner` | `String` | Owner's username |
| `plate` | `String` | Licence plate |
| `texture` | `Option<String>` | Livery, if any |
| `color_hex` | `String` | e.g. `"#ff4444"` |
| `color_name` | `String` | e.g. `"Super Red"` |

## Bans

`bans()` calls `GET /v1/server/bans`, the only v1 endpoint with no v2 equivalent. It maps
Roblox user IDs to usernames:

```rust,no_run
async fn list_bans(client: &erlc::Client) -> erlc::Result<()> {
    for (id, name) in client.bans().await? {
        println!("{name} ({id})");
    }
    Ok(())
}
```

## Storing and forwarding data

Every model implements `serde::Serialize` and `Deserialize` using the original ER:LC field
names (`"Player"`, `"WantedStars"`, ...), so you can cache responses or forward them to a
frontend unchanged:

```rust,no_run
async fn to_json(client: &erlc::Client) -> Result<String, Box<dyn std::error::Error>> {
    let players = client.players().await?;
    Ok(serde_json::to_string(&players)?)
}
```

Unknown fields sent by the API are ignored, so new API fields do not break your code.
Models are `#[non_exhaustive]`: read their fields freely, but build them by deserializing
rather than with struct literals.
