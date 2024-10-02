use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Insertable, Queryable, Selectable, Serialize, Debug, Clone)]
#[diesel(table_name = crate::schema::teams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Team {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub name: String,
}

#[derive(Insertable, Queryable, Selectable, Serialize, Debug, Clone)]
#[diesel(table_name = crate::schema::team_members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TeamMember {
    pub team_id: i32,
    pub student_id: i32,
    pub is_leader: bool,
}

#[derive(Insertable, Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::team_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TeamRequest {
    pub team_id: i32,
    pub student_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewTeamRequest {
    pub team_id: i32,
    pub email: String,
}
