use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::forms::users::Profile;

use super::{students::Department, users::User};

#[derive(diesel_derive_enum::DbEnum, Debug, Clone, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::Title"]
#[allow(non_camel_case_types)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum Title {
    PROF,
    ASOCP,
    ASP,
    GUEST,
}

#[derive(Insertable, Queryable, Selectable, Serialize, Debug, Clone)]
#[diesel(table_name = crate::schema::faculty)]
#[diesel(belongs_to(User, foreign_key=user_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Faculty {
    pub user_id: i32,
    pub title: Title,
    pub dept: Department,
}

#[derive(Serialize, Debug, Clone)]
pub struct FacultyResponse {
    #[serde(flatten)]
    pub faculty: Faculty,
    #[serde(flatten)]
    pub profile: Profile,
}
