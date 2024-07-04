PRAGMA foreign_keys=OFF;

CREATE TABLE new_tokens (
    value CHAR(36) PRIMARY KEY NOT NULL,
    owner_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    lifetime FLOAT NULL,
    creation_time DATETIME NOT NULL
);

INSERT INTO new_tokens SELECT * FROM tokens;

DROP TABLE tokens;

ALTER TABLE new_tokens RENAME TO tokens;

PRAGMA foreign_keys=ON;
