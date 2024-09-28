-- Your SQL goes here
CREATE TABLE workshops (
	id SERIAL PRIMARY KEY,
	name TEXT NOT NULL,
	description TEXT NOT NULL,
	mode MODE NOT NULL,
	venue TEXT NOT NULL,
	domain_id INTEGER REFERENCES domains(id) NOT NULL,
	points INTEGER NOT NULL,
	ps_link TEXT NOT NULL,
	prof_name TEXT NOT NULL,
	prof_title TEXT NOT NULL,
	start_time TIMESTAMP NOT NULL,
	end_time TIMESTAMP NOT NULL,
	registeration_start TIMESTAMP NOT NULL,
	registeration_end TIMESTAMP NOT NULL,
	whatsapp_link TEXT NOT NULL,
	photo_hash BYTEA

)
