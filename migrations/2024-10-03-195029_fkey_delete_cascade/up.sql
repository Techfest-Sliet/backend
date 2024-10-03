-- Your SQL goes here
ALTER TABLE payments
  DROP CONSTRAINT payments_user_id_fkey,
  ADD CONSTRAINT payments_user_id_fkey FOREIGN KEY (user_id)
      REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE faculty
  DROP CONSTRAINT faculty_user_id_fkey,
  ADD CONSTRAINT faculty_user_id_fkey FOREIGN KEY (user_id)
      REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE team_members
  DROP CONSTRAINT team_members_team_id_fkey,
  ADD CONSTRAINT team_members_team_id_fkey FOREIGN KEY (team_id)
      REFERENCES users(id) ON DELETE CASCADE;
