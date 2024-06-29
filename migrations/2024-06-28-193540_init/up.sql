CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username VARCHAR(25) NULL,
    hashed_password CHAR(128) NULL,
    creation_time DATETIME NOT NULL,
    properties TEXT NULL,
    is_deleted BOOL NOT NULL DEFAULT FALSE
);


CREATE TABLE tokens (
    value CHAR(36) PRIMARY KEY NOT NULL,
    owner_id INTEGER REFERENCES users(id),
    lifetime FLOAT NULL,
    creation_time DATETIME NOT NULL
);


CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    sender_id INTEGER REFERENCES users(id) NOT NULL,
    receiver_id INTEGER REFERENCES users(id) NOT NULL,
    body TEXT NOT NULL,
    replying_id INTEGER REFERENCES messages(id) NOT NULL,
    sent_time DATETIME NOT NULL,
    is_read BOOL NOT NULL DEFAULT FALSE,
    is_deleted BOOL NOT NULL DEFAULT FALSE
);
