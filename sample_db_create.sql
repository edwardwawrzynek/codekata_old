PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE __diesel_schema_migrations (version VARCHAR(50) PRIMARY KEY NOT NULL,run_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP);
INSERT INTO __diesel_schema_migrations VALUES('20200922214857','2020-10-01 02:57:58');
INSERT INTO __diesel_schema_migrations VALUES('20200925153646','2020-10-01 05:11:19');
INSERT INTO __diesel_schema_migrations VALUES('20201001073652','2020-10-01 07:38:43');
CREATE TABLE users (
                       id INTEGER NOT NULL PRIMARY KEY,
                       username TEXT NOT NULL,
                       display_name TEXT NOT NULL,
                       password_hash TEXT NOT NULL,
                       api_key_hash TEXT
);
INSERT INTO users VALUES(1, 'api1@example.com', 'API Key 1', '$2b$08$lHQNXr8RtP5CfkUcX66N0eBqCHIfDUHpOUhRfWNgEIBcg9O8r2xdi', 'c7c3dc01bbe73cb7643d9762ea264762f700a743eafa518c287a517a3f035092');
INSERT INTO users VALUES(2, 'api2@example.com', 'API Key 2', '$2b$08$lHQNXr8RtP5CfkUcX66N0eBqCHIfDUHpOUhRfWNgEIBcg9O8r2xdi', '9682569c3a671a8ad261b1f05987955545787379afe9bb0b867b792818226ed1');
INSERT INTO users VALUES(3, 'example@example.com', 'Human Player', '$2b$08$lHQNXr8RtP5CfkUcX66N0eBqCHIfDUHpOUhRfWNgEIBcg9O8r2xdi', NULL);
CREATE TABLE db_games (
                          id INTEGER NOT NULL PRIMARY KEY,
                          title VARCHAR NOT NULL,
                          state TEXT,
                          owner_id INTEGER NOT NULL,
                          players VARCHAR NOT NULL,
                          active INTEGER NOT NULL
);
COMMIT;