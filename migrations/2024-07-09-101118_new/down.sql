PRAGMA foreign_keys = OFF;

CREATE TABLE old_users (
	id INTEGER PRIMARY KEY,
	username VARCHAR,
	hashed_password VARCHAR,
	creation_time DATETIME,
	properties VARCHAR,
	is_deleted BOOLEAN
);


CREATE TABLE old_tokens (
	value VARCHAR PRIMARY KEY NOT NULL,
	owner_id INTEGER REFERENCES users(id),
	lifetime FLOAT,
	creation_time DATETIME
);


CREATE TABLE old_messages (
	id INTEGER PRIMARY KEY NOT NULL,
	sender_id INTEGER REFERENCES users(id),
	receiver_id INTEGER REFERENCES users(id),
	body VARCHAR,
	replying_id INTEGER REFERENCES messages(id),
	sent_time DATETIME,
	is_read BOOLEAN,
	is_deleted BOOLEAN
);


INSERT INTO old_users SELECT * FROM users;
INSERT INTO old_tokens SELECT * FROM tokens;
INSERT INTO old_messages SELECT * FROM messages;


DROP TABLE users;
DROP TABLE tokens;
DROP TABLE messages;


ALTER TABLE old_users RENAME TO users;
ALTER TABLE old_tokens RENAME TO tokens;
ALTER TABLE old_messages RENAME TO messages;


PRAGMA foreign_keys = ON;
