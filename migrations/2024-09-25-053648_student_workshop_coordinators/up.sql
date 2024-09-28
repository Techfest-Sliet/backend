-- Your SQL goes here
CREATE TABLE student_workshop_coordinators (
	student_id INTEGER REFERENCES students(user_id),
	workshop_id INTEGER REFERENCES workshops(id),
	PRIMARY KEY (student_id, workshop_id)
)
