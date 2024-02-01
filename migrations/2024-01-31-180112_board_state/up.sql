-- Your SQL goes here
CREATE TABLE board_states (
    id                TEXT PRIMARY KEY,
    is_ready_to_solve BOOLEAN NOT NULL,
    is_solved         BOOLEAN NOT NULL,
    blocks            TEXT NOT NULL,
    filled            TEXT NOT NULL,
    moves             TEXT NOT NULL
)
