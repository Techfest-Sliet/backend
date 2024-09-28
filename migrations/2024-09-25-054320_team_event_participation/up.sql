-- Your SQL goes here
CREATE TABLE team_event_participations (
	team_id INTEGER REFERENCES teams(id) NOT NULL,
	event_id INTEGER REFERENCES events(id) NOT NULL,
	attended boolean NOT NULL DEFAULT false,
	PRIMARY KEY(team_id, event_id)
)
