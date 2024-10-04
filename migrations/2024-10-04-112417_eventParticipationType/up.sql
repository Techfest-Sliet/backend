-- Your SQL goes here
CREATE TYPE PARTICIPATION_TYPE AS ENUM ('INDIVIDUAL', 'TEAM');
ALTER TABLE events ADD COLUMN participation_type PARTICIPATION_TYPE NOT NULL DEFAULT 'INDIVIDUAL';
