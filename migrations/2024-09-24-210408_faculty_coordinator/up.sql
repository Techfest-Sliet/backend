-- Your SQL goes here
CREATE TABLE faculty_coordinators (
	faculty_id INTEGER REFERENCES faculty(user_id) NOT NULL,
	domain_id INTEGER REFERENCES domains(id) NOT NULL,
	PRIMARY KEY(faculty_id, domain_id)
)
