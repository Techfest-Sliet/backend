use diesel::prelude::*;
use serde::Serialize;
#[derive(Insertable, Queryable, Selectable, Serialize, Debug, Clone)]
#[diesel(table_name = crate::schema::domains)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Domain {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub name: String,
    pub description: String,
    pub photo_hash: Option<Vec<u8>>,
}
