-- Your SQL goes here
CREATE TABLE users (
	id SERIAL PRIMARY KEY,
	name TEXT NOT NULL,
	dob DATE NOT NULL,
	email TEXT NOT NULL UNIQUE,
	phone TEXT NOT NULL,
	role ROLE NOT NULL,
	photo_hash BYTEA,
	verified boolean NOT NULL DEFAULT false,
	password_hash TEXT NOT NULL
)
