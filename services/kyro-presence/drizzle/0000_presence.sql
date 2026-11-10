CREATE TABLE sessions (
 token_hash TEXT PRIMARY KEY NOT NULL,
 uuid TEXT NOT NULL,
 name TEXT NOT NULL,
 expires INTEGER NOT NULL
);
--> statement-breakpoint
CREATE INDEX sessions_uuid_expires ON sessions (uuid,expires);
--> statement-breakpoint
CREATE INDEX sessions_expires ON sessions (expires);
--> statement-breakpoint
CREATE TABLE rate_limits (
 bucket TEXT PRIMARY KEY NOT NULL,
 count INTEGER NOT NULL,
 expires INTEGER NOT NULL
);
--> statement-breakpoint
CREATE INDEX rate_limits_expires ON rate_limits (expires);
