CREATE TABLE db_games (
  id INTEGER NOT NULL PRIMARY KEY,
  title VARCHAR NOT NULL,
  state TEXT,
  owner_id INTEGER NOT NULL,
  players VARCHAR NOT NULL,
  active INTEGER NOT NULL
)