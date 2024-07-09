CREATE TABLE IF NOT EXISTS users (
	id INTEGER PRIMARY KEY,
	username VARCHAR,
	hashed_password VARCHAR,
	creation_time DATETIME,
	properties VARCHAR,
	is_deleted BOOLEAN
);


CREATE TABLE IF NOT EXISTS tokens (
	value VARCHAR PRIMARY KEY NOT NULL,
	owner_id INTEGER REFERENCES users(id),
	lifetime FLOAT,
	creation_time DATETIME
);


CREATE TABLE IF NOT EXISTS messages (
	id INTEGER PRIMARY KEY NOT NULL,
	sender_id INTEGER REFERENCES users(id),
	receiver_id INTEGER REFERENCES users(id),
	body VARCHAR,
	replying_id INTEGER REFERENCES messages(id),
	sent_time DATETIME,
	is_read BOOLEAN,
	is_deleted BOOLEAN
)