
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

UPDATE students SET dept = 'CS' WHERE dept='EIE';
UPDATE faculty SET dept = 'CS' WHERE dept='EIE';

ALTER TABLE students 
  ALTER COLUMN dept TYPE text ;

ALTER TABLE students 
  ALTER COLUMN dept TYPE DEPARTMENT_NEW USING dept::DEPARTMENT_NEW;

ALTER TABLE faculty 
  ALTER COLUMN dept TYPE text ;

ALTER TABLE faculty 
  ALTER COLUMN dept TYPE DEPARTMENT_NEW USING dept::DEPARTMENT_NEW ;

DROP TYPE DEPARTMENT;

ALTER TYPE DEPARTMENT_NEW RENAME TO DEPARTMENT;
