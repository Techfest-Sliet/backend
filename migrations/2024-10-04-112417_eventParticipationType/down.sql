-- This file should undo anything in `up.sql`
ALTER TABLE events DROP COLUMN participation_type;
DROP TYPE PARTICIPATION_TYPE;
