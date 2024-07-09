PRAGMA foreign_keys = OFF;

CREATE TABLE new_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username VARCHAR(25) NULL,
    hashed_password CHAR(128) NULL,
    creation_time DATETIME NOT NULL,
    properties TEXT NULL,
    is_deleted BOOL NOT NULL DEFAULT FALSE
);


CREATE TABLE new_tokens (
    value CHAR(36) PRIMARY KEY NOT NULL,
    owner_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    lifetime FLOAT NULL,
    creation_time DATETIME NOT NULL
);


CREATE TABLE new_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    sender_id INTEGER REFERENCES users(id) NOT NULL,
    receiver_id INTEGER REFERENCES users(id) NOT NULL,
    body TEXT NULL,
    replying_id INTEGER REFERENCES messages(id) NULL,
    sent_time DATETIME NOT NULL,
    is_read BOOL NULL DEFAULT FALSE,
    is_deleted BOOL NOT NULL DEFAULT FALSE
);


INSERT INTO new_users SELECT * FROM users;
INSERT INTO new_tokens SELECT * FROM tokens;
INSERT INTO new_messages SELECT * FROM messages;


DROP TABLE users;
DROP TABLE tokens;
DROP TABLE messages;


ALTER TABLE new_users RENAME TO users;
ALTER TABLE new_tokens RENAME TO tokens;
ALTER TABLE new_messages RENAME TO messages;


PRAGMA foreign_keys = ON;
