-- Your SQL goes here

CREATE TABLE sponsors (
	id SERIAL PRIMARY KEY,
	name TEXT NOT NULL,
	photo_hash BYTEA NOT NULL
)
