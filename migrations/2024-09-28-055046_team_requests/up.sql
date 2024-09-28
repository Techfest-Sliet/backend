-- Your SQL goes here
CREATE TABLE team_requests (
	team_id INTEGER REFERENCES teams(id) NOT NULL,
	student_id INTEGER REFERENCES students(user_id) NOT NULL,
	PRIMARY KEY(team_id, student_id)
)
