-- Your SQL goes here
CREATE TABLE domains (
	id SERIAL PRIMARY KEY,
	name TEXT NOT NULL,
	description TEXT NOT NULL,
	photo_hash BYTEA
)
