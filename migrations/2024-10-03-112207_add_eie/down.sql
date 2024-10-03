-- This file should undo anything in `up.sql`

CREATE TYPE DEPARTMENT_NEW AS ENUM (
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

UPDATE FROM users WHERE role='EIE' SET role = "CS";

-- Convert to new type, casting via text representation
ALTER TABLE users 
  ALTER COLUMN role TYPE DEPARTMENT_NEW 
    USING (power::text::DEPARTMENT_NEW);

-- and swap the types
DROP TYPE DEPARTMENT;

ALTER TYPE DEPARTMENT_NEW RENAME TO DEPARTMENT;
