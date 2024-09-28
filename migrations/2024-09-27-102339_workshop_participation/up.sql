-- Your SQL goes here
CREATE TABLE workshop_participation (
	workshop_id INTEGER REFERENCES workshops(id) NOT NULL,
	user_id INTEGER REFERENCES users(id) NOT NULL,
	attended boolean NOT NULL DEFAULT false,
	PRIMARY KEY(workshop_id, user_id)
)
