# Troubleshooting

## `invalid input: server key must not be empty`

The environment variable is missing or empty. Check `ERLC_SERVER_KEY` is set in the same
shell that runs your program.

## HTTP 403 with code 2002

The key is invalid or expired. The server owner probably regenerated it: get the new key
from **Settings → ER:LC API → API Key**. Stop using the old key; repeated 403s get your app
temporarily blocked.

## Code 2005: unauthenticated requests for multiple servers

Your app calls the API for more than one server without a global key. Register a public
application and set `ClientBuilder::global_key`. See [Public applications](public-apps.md).

## Code 9999: module out of date

The ER:LC module on the private server is outdated. Kick all players from the server, then
try again.

## HTTP 422 or code 3002 on commands

The server has no players. Commands only work while at least one player is in the server.

## `Error::RateLimited` with `is_local() == true`

The client blocked the request itself because an earlier response put that bucket on hold.
Wait for `retry_after()`. See [Rate limits](rate-limits.md).

## An HTML page or Cloudflare block

The ER:LC API blocks clients that persistently ignore rate limits. Open an API ticket in the
ER:LC Discord server. The client never uses HTML bodies as error messages; inspect
`ApiError::body()` to see the page.

## `request to the ER:LC API timed out`

The API or the in-game server was slow. Retry reads later, or raise the limit with
`ClientBuilder::timeout`. For commands, check the command log before resending: the command
may have run.

## TLS or connection errors at startup

If you disabled default features, enable `rustls` or `native-tls`; without one the client
cannot make HTTPS requests. Behind a corporate proxy, pass a configured
`erlc::reqwest::Client` with `ClientBuilder::http_client`.

## `could not decode ER:LC API response`

The API returned data in a shape the models do not expect, usually because the API changed.
Please [open an issue](https://github.com/ERLC-Rust/erlc.rs/issues) with the redacted
`DecodeError::body()`. Remove player names, IDs and the join key first.

## Still stuck?

- [ER:LC API documentation](https://apidocs.erlc.gg/)
- [Open an issue](https://github.com/ERLC-Rust/erlc.rs/issues/new/choose); never include keys
