use std::fmt::Display;

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
    EIE,
    FET,
    MECH,
    DS,
    MH,
    PHY,
    MATHS,
    CHM,
}

impl Department {
    pub const VARIANTS: [Self; 12] = [
        Self::CS,
        Self::CT,
        Self::CEN,
        Self::ECE,
        Self::EIE,
        Self::FET,
        Self::MECH,
        Self::DS,
        Self::MH,
        Self::PHY,
        Self::MATHS,
        Self::CHM,
    ];
}

impl Display for Department {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Department::CS => "Computer Science",
            Department::CT => "Chemical Technology",
            Department::CEN => "Civil Engineering",
            Department::ECE => "Electronics and Communication Engineering",
            Department::EIE => "Instrumentation Engineering",
            Department::FET => "Food Engineering and Technology",
            Department::MECH => "Mechanical Engineering",
            Department::DS => "Disability Studies",
            Department::MH => "Management and Humanities",
            Department::PHY => "Physics",
            Department::MATHS => "Maths",
            Department::CHM => "Chemistry",
        })
    }
}

#[derive(Insertable, Queryable, Selectable, Serialize, Debug, Clone)]
#[diesel(table_name = crate::schema::students)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Student {
    pub user_id: i32,
    pub college: String,
    pub reg_no: String,
    pub dept: Department,
}

#[derive(Serialize, Debug, Clone)]
pub struct StudentResponse {
    #[serde(flatten)]
    pub student: Student,
    #[serde(flatten)]
    pub profile: Profile,
}
