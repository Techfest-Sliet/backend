use diesel::prelude::*;
use serde::Deserialize;

#[derive(Insertable, Queryable, Selectable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::payments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Payment {
    user_id: i32,
    payment_id: String,
    payment_amount: i32,
    #[serde(skip)]
    verified: bool,
}
