CREATE TABLE IF NOT EXISTS master_webhooks
(
    id      INTEGER PRIMARY KEY NOT NULL,
    server_name    TEXT          NOT NULL,
    guild_id     TEXT    UNIQUE     NOT NULL,
    webhook_url     TEXT                NOT NULL
);


CREATE TABLE IF NOT EXISTS member_webhooks
(
    a_member_id      TEXT       NOT NULL,
    b_server_name    TEXT          NOT NULL,
    b_guild_id     TEXT     NOT NULL,
    b_channel_id     TEXT     NOT NULL,
    b_webhook_url     TEXT                NOT NULL,

    PRIMARY KEY(a_member_id, b_guild_id)
);

CREATE TABLE IF NOT EXISTS a_member_times_data
(
    member_id      TEXT PRIMARY KEY NOT NULL,
    member_name    TEXT          NOT NULL,
    channel_id     TEXT     NOT NULL,
    webhook_url     TEXT                NOT NULL
);

-- この程度だったらほんとはkvsとか使いたい
CREATE TABLE IF NOT EXISTS a_server_data
(
    guild_id TEXT PRIMARY KEY NOT NULL,
    server_name TEXT  NOT NULL,
    master_channel_id TEXT NOT NULL,
    master_webhook_url TEXT NOT NULL,
    private_key_pem TEXT NOT NULL,
    public_key_pem TEXT NOT NULL
);