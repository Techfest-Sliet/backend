-- Your SQL goes here
CREATE TABLE individual_event_participation (
	event_id INTEGER REFERENCES events(id) NOT NULL,
	user_id INTEGER REFERENCES users(id) NOT NULL,
	attended boolean NOT NULL DEFAULT false,
	PRIMARY KEY(event_id, user_id)
)
