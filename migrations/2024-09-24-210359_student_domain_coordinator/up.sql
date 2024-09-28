-- Your SQL goes here
CREATE TABLE student_domain_coordinators (
	student_id INTEGER REFERENCES students(user_id) NOT NULL,
	domain_id INTEGER REFERENCES domains(id) NOT NULL,
	PRIMARY KEY(student_id, domain_id)
)
