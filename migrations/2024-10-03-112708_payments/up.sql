-- Your SQL goes here
CREATE TABLE payments (
	user_id INTEGER REFERENCES users(id) NOT NULL,
	payment_id TEXT NOT NULL PRIMARY KEY,
	payment_amount INTEGER NOT NULL,
	verified BOOLEAN NOT NULL DEFAULT FALSE
)
