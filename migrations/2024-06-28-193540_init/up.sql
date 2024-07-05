CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username VARCHAR(25) NULL,
    hashed_password CHAR(128) NULL,
    creation_time DATETIME NOT NULL,
    properties TEXT NULL,
    is_deleted BOOL NOT NULL DEFAULT FALSE
);


CREATE TABLE IF NOT EXISTS tokens (
    value CHAR(36) PRIMARY KEY NOT NULL,
    owner_id INTEGER REFERENCES users(id) NOT NULL,
    lifetime FLOAT NULL,
    creation_time DATETIME NOT NULL
);


CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    sender_id INTEGER REFERENCES users(id) NOT NULL,
    receiver_id INTEGER REFERENCES users(id) NOT NULL,
    body TEXT NULL,
    replying_id INTEGER REFERENCES messages(id) NULL,
    sent_time DATETIME NOT NULL,
    is_read BOOL NULL DEFAULT FALSE,
    is_deleted BOOL NOT NULL DEFAULT FALSE
);
