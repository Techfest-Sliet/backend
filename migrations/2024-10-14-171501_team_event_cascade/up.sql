-- Your SQL goes here
ALTER TABLE team_event_participations
  DROP CONSTRAINT team_event_participations_team_id_fkey,
  ADD CONSTRAINT team_event_participations_team_id_fkey FOREIGN KEY (team_id)
      REFERENCES teams(id) ON DELETE CASCADE;

ALTER TABLE team_requests
  DROP CONSTRAINT team_requests_team_id_fkey,
  ADD CONSTRAINT team_requests_team_id_fkey FOREIGN KEY (team_id)
      REFERENCES teams(id) ON DELETE CASCADE;
