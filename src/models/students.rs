use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::forms::users::Profile;
#[derive(diesel_derive_enum::DbEnum, Debug, Clone, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::Department"]
#[allow(non_camel_case_types)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum Department {
    	CS,
	CT,
	CEN,
	ECE,
	FET,
	MECH,
	DS,
	MH,
	PHY,
	MATHS,
	CHM
}

#[derive(Insertable, Queryable, Selectable, Serialize, Debug, Clone)]
#[diesel(table_name = crate::schema::students)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Student {
    pub user_id: i32,
    pub college: String,
    pub reg_no: String,
    pub dept: Department
}

#[derive(Serialize, Debug, Clone)]
pub struct StudentResponse {
    #[serde(flatten)]
    pub student: Student,
    #[serde(flatten)]
    pub profile: Profile,
}
