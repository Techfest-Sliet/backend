-- Your SQL goes here
CREATE TABLE team_members (
	team_id INTEGER REFERENCES teams(id) NOT NULL,
	student_id INTEGER REFERENCES students(user_id) NOT NULL,
	is_leader BOOLEAN NOT NULL DEFAULT false,
	PRIMARY KEY(team_id, student_id)
)
