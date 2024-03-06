-- Your SQL goes here
CREATE TABLE solutions (
    id     SERIAL PRIMARY KEY,
    hash   BIGINT NOT NULL,
    moves  TEXT
)
