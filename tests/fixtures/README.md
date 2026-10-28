# Test fixtures

JSON bodies used by the tests. They are built from the examples in the official
ER:LC OpenAPI specs (`https://api.erlc.gg/internal/docs/apispec.v2.json`) unless a
file name says otherwise. Files starting with `live_` are redacted captures from a real server.

Fixtures captured from a live server must be redacted before they are committed:
replace player names, Roblox IDs, the server name, owner IDs and the join key with
placeholder values. Keep raw captures in `tests/fixtures/raw/`, which is ignored by git.
