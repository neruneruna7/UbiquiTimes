CREATE TABLE IF NOT EXISTS serverwebhooks
(
    id      INTEGER PRIMARY KEY NOT NULL,
    servername    TEXT    UNIQUE      NOT NULL,
    -- guildid    INTEGER          NOT NULL,
    webhookurl     TEXT                NOT NULL
);


CREATE TABLE IF NOT EXISTS privatewebhooks
(
    id      INTEGER PRIMARY KEY NOT NULL,
    servername    TEXT          NOT NULL,
    userid    INTEGER          NOT NULL,
    webhookurl     TEXT                NOT NULL
);