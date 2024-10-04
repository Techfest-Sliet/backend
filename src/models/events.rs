use diesel::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Insertable, Queryable, Selectable, Serialize, Debug, Clone)]
#[diesel(table_name = crate::schema::events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Event {
    #[diesel(skip_insertion)]
    pub id: i32,
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
    pub participation_type: ParticipationType,
    pub photo_hash: Option<Vec<u8>>,
}

#[derive(diesel_derive_enum::DbEnum, Debug, Clone, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::ParticipationType"]
#[allow(non_camel_case_types)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum ParticipationType {
    INDIVIDUAL,
    TEAM
}

#[derive(diesel_derive_enum::DbEnum, Debug, Clone, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::Mode"]
#[allow(non_camel_case_types)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum Mode {
    ONLINE,
    HYBRID,
    OFFLINE,
}
