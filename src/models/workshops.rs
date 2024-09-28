use diesel::prelude::*;
use serde::Serialize;

use super::events::Mode;

#[derive(Insertable, Queryable, Selectable, Serialize, Debug, Clone)]
#[diesel(table_name = crate::schema::workshops)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Workshop {
    #[diesel(skip_insertion)]
    pub id: i32,
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
    pub photo_hash: Option<Vec<u8>>
}
