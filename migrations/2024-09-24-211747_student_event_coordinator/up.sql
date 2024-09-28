-- Your SQL goes here
CREATE TABLE student_event_coordinators (
	student_id INTEGER REFERENCES students(user_id) NOT NULL,
	event_id INTEGER REFERENCES events(id) NOT NULL,
	PRIMARY KEY(student_id, event_id)
)
