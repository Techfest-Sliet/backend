-- Your SQL goes here
ALTER TABLE individual_event_participation
  DROP CONSTRAINT individual_event_participation_event_id_fkey,
  ADD CONSTRAINT individual_event_participation_event_id_fkey FOREIGN KEY (event_id)
      REFERENCES events(id) ON DELETE CASCADE;

ALTER TABLE individual_event_participation
  DROP CONSTRAINT individual_event_participation_user_id_fkey,
  ADD CONSTRAINT individual_event_participation_user_id_fkey FOREIGN KEY (user_id)
      REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE workshop_participation
  DROP CONSTRAINT workshop_participation_workshop_id_fkey,
  ADD CONSTRAINT workshop_participation_workshop_id_fkey FOREIGN KEY (workshop_id)
      REFERENCES workshops(id) ON DELETE CASCADE;

ALTER TABLE workshop_participation
  DROP CONSTRAINT workshop_participation_user_id_fkey,
  ADD CONSTRAINT workshop_participation_user_id_fkey FOREIGN KEY (user_id)
      REFERENCES users(id) ON DELETE CASCADE;
