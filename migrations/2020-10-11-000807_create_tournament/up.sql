CREATE TABLE tournaments (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    players INTEGER[] NOT NULL,
    games INTEGER[],
    owner_id INTEGER NOT NULL
)