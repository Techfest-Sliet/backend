-- Your SQL goes here
CREATE TYPE ROLE AS ENUM (
	'SUPER_ADMIN',
	'FACULTY_COORDINATOR',
	'STUDENT_COORDINATOR',
	'PARTICIPANT'
);

CREATE TYPE DEPARTMENT AS ENUM (
	'CS',
	'CT',
	'CEN',
	'ECE',
	'FET',
	'MECH',
	'DS',
	'MH',
	'PHY',
	'MATHS',
	'CHM'
);

CREATE TYPE TITLE AS ENUM (
	'PROF',
	'ASOCP',
	'ASP',
	'GUEST'
);

CREATE TYPE MODE AS ENUM (
	'ONLINE',
	'OFFLINE',
	'HYBRID'
);
