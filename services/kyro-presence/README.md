# KYRO Presence

Short-lived cosmetic client status for Minecraft name badges. Each running KYRO
game registers a UUID and nickname, renews its lease every 20 seconds, and queries
only UUIDs present in its player list. No game access token, server address, chat,
password, or permanent installation identifier is collected.

`POST /v1/sync`: `{uuid,name,lease?,peers:[uuid,...]}` returns a session lease and
active queried players. Leases expire after 75 seconds; `POST /v1/leave` removes
one early. There is no endpoint listing all players. Session tokens are stored
hashed, and expired sessions are removed in bounded batches. This is a cosmetic
client claim, not proof of account ownership.

The Worker uses a D1 binding named `DB`. Its schema is managed by `drizzle/`;
requests never create database tables. `GET /health` checks the migrated database.

Run `node --test test/*.test.mjs`, then `node scripts/build.mjs` and
`node scripts/validate-artifact.mjs`. Tests execute the production SQL against
SQLite and cover mutual discovery, token binding, expiry, leaving, request bounds,
and Worker entrypoint behavior.

Deploy with the Sites source workflow using this directory's retained project ID.
The desktop client requires an endpoint accessible without a browser login.
