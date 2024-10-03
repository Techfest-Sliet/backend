use serde::Deserialize;

use diesel::prelude::*;

use crate::models::events::Mode;
#[derive(Deserialize, Insertable, Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateEvent {
    pub name: String,
    pub description: String,
    pub mode: Mode,
    pub venue: String,
    pub domain_id: i32,
    pub prize: i32,
    pub points: i32,
    pub ps_link: String,
    pub start_time: chrono::NaiveDateTime,
    pub end_time: chrono::NaiveDateTime,
    pub registeration_start: chrono::NaiveDateTime,
    pub registeration_end: chrono::NaiveDateTime,
    pub whatsapp_link: String,
}

#[derive(Queryable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeleteEvent {
    pub id: i32,
}

#[derive(Deserialize, AsChangeset, Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChangeEvent {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub mode: Option<Mode>,
    pub venue: Option<String>,
    pub prize: Option<i32>,
    pub points: Option<i32>,
    pub ps_link: Option<String>,
    pub start_time: Option<chrono::NaiveDateTime>,
    pub end_time: Option<chrono::NaiveDateTime>,
    pub registeration_start: Option<chrono::NaiveDateTime>,
    pub registeration_end: Option<chrono::NaiveDateTime>,
    pub whatsapp_link: Option<String>,
}

#[derive(Queryable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EventId {
    pub id: i32,
}

#[derive(Queryable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GetEventStudentCoordinator {
    pub id: i32,
}

#[derive(Queryable, Insertable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::student_event_coordinators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AddEventStudentCoordinator {
    pub student_id: i32,
    pub event_id: i32,
}

#[derive(Queryable, Insertable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::individual_event_participation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EventIndividualAttendance {
    pub user_id: i32,
    pub event_id: i32,
}

#[derive(Queryable, Insertable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::team_event_participations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EventTeamAttendance {
    pub team_id: i32,
    pub event_id: i32,
}
