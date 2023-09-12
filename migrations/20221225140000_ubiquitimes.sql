CREATE TABLE IF NOT EXISTS master_webhooks
(
    id      INTEGER PRIMARY KEY NOT NULL,
    server_name    TEXT          NOT NULL,
    guild_id    INTEGER     UNIQUE     NOT NULL,
    webhook_url     TEXT                NOT NULL
);


CREATE TABLE IF NOT EXISTS member_webhooks
(
    id      INTEGER PRIMARY KEY NOT NULL,
    server_name    TEXT          NOT NULL,
    member_id    INTEGER          NOT NULL,
    channel_id  TEXT NOT NULL,
    webhook_url     TEXT                NOT NULL
);