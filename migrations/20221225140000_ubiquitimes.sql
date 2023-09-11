CREATE TABLE IF NOT EXISTS master_webhooks
(
    id      INTEGER PRIMARY KEY NOT NULL,
    server_name    TEXT    UNIQUE      NOT NULL,
    -- guildid    INTEGER          NOT NULL,
    webhook_url     TEXT                NOT NULL
);


CREATE TABLE IF NOT EXISTS member_webhooks
(
    id      INTEGER PRIMARY KEY NOT NULL,
    server_name    TEXT          NOT NULL,
    user_id    INTEGER          NOT NULL,
    webhook_url     TEXT                NOT NULL
);