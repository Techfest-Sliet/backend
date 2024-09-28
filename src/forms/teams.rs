use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use serde::Deserialize;

#[derive(Deserialize, Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::teams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TeamId {
    pub id: i32
}

#[derive(Deserialize, Insertable, Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::teams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TeamName {
    pub name: String
}

#[derive(Deserialize, AsChangeset, Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::teams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChangeTeam {
    pub id: i32,
    pub name: Option<String>
}

#[derive(Insertable, Queryable, Selectable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::team_members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MemberId {
    pub team_id: i32,
    pub student_id: i32,
}
