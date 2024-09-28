-- Your SQL goes here
CREATE TABLE faculty (
	user_id INTEGER PRIMARY KEY REFERENCES users(id),
	dept DEPARTMENT NOT NULL,
	title TITLE NOT NULL
)
