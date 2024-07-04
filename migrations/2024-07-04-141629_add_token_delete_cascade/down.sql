PRAGMA foreign_keys=OFF;

BEGIN;

CREATE TABLE new_tokens (
    value CHAR(36) PRIMARY KEY NOT NULL,
    owner_id INTEGER REFERENCES users(id) NOT NULL,
    lifetime FLOAT NULL,
    creation_time DATETIME NOT NULL
);

INSERT INTO new_tokens VALUES * FROM tokens;

DROP TABLE tokens;

ALTER TABLE new_tokens RENAME TO tokens;

COMMIT;

PRAGMA foreign_keys=ON;

