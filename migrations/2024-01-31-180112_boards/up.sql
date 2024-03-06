-- Your SQL goes here
CREATE TABLE boards (
    id     SERIAL PRIMARY KEY,
    state  VARCHAR(20) NOT NULL,
    blocks TEXT NOT NULL,
    grid   TEXT NOT NULL,
    moves  TEXT NOT NULL
)
