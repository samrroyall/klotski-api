-- Your SQL goes here
CREATE TABLE boards (
    id     SERIAL PRIMARY KEY,
    state  VARCHAR(20) NOT NULL,
    blocks TEXT NOT NULL,
    filled TEXT NOT NULL,
    moves  TEXT NOT NULL,
    next_moves  TEXT NOT NULL
)
