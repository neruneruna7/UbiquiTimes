CREATE TABLE IF NOT EXISTS master_webhooks
(
    id      INTEGER PRIMARY KEY NOT NULL,
    server_name    TEXT          NOT NULL,
    guild_id     TEXT    UNIQUE     NOT NULL,
    webhook_url     TEXT                NOT NULL
);


CREATE TABLE IF NOT EXISTS member_webhooks
(
    id      INTEGER PRIMARY KEY NOT NULL,
    server_name    TEXT          NOT NULL,
    member_id      TEXT       NOT NULL,
    channel_id     TEXT     NOT NULL,
    webhook_url     TEXT                NOT NULL
);

CREATE TABLE IF NOT EXISTS member_times_data
(
    member_id      TEXT PRIMARY KEY NOT NULL,
    member_name    TEXT          NOT NULL,
    channel_id     TEXT     NOT NULL,
    webhook_url     TEXT                NOT NULL
);
-- この程度だったらほんとはkvsとか使いたい
CREATE TABLE IF NOT EXISTS server_data
(
    guild_id TEXT PRIMARY KEY NOT NULL,
    server_name TEXT,
    server_master_channel_id TEXT
);