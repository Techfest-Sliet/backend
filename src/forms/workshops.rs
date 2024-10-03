use serde::Deserialize;

use diesel::prelude::*;

use crate::models::events::Mode;
#[derive(Deserialize, Insertable, Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::workshops)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateWorkshop {
    pub name: String,
    pub description: String,
    pub mode: Mode,
    pub venue: String,
    pub domain_id: i32,
    pub points: i32,
    pub ps_link: String,
    pub start_time: chrono::NaiveDateTime,
    pub end_time: chrono::NaiveDateTime,
    pub registeration_start: chrono::NaiveDateTime,
    pub registeration_end: chrono::NaiveDateTime,
    pub whatsapp_link: String,
}

#[derive(Queryable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::workshops)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeleteWorkshop {
    pub id: i32,
}

#[derive(Deserialize, AsChangeset, Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::workshops)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChangeWorkshop {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub mode: Option<Mode>,
    pub venue: Option<String>,
    pub ps_link: Option<String>,
    pub start_time: Option<chrono::NaiveDateTime>,
    pub end_time: Option<chrono::NaiveDateTime>,
    pub registeration_start: Option<chrono::NaiveDateTime>,
    pub registeration_end: Option<chrono::NaiveDateTime>,
    pub whatsapp_link: Option<String>,
}

#[derive(Queryable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::workshops)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkshopId {
    pub id: i32,
}

#[derive(Queryable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::workshops)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GetWorkshopStudentCoordinator {
    pub id: i32,
}

#[derive(Queryable, Insertable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::student_workshop_coordinators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AddWorkshopStudentCoordinator {
    pub student_id: i32,
    pub workshop_id: i32,
}

#[derive(Queryable, Insertable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::workshop_participation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkshopIndividualAttendance {
    pub user_id: i32,
    pub workshop_id: i32,
}
