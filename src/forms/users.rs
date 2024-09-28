use std::hash::{DefaultHasher, Hash, Hasher};

use diesel::{AsChangeset, Queryable, Selectable};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::models::users::Role;

#[derive(Deserialize)]
pub struct SignInForm {
    pub email: String,
    pub password: String,
}

#[derive(Selectable, Queryable, Debug, Clone, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Profile {
    id: i32,
    dob: chrono::NaiveDate,
    name: String,
    email: String,
    phone: String,
    role: Role,
    verified: bool,
}

#[derive(Queryable, AsChangeset, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChangeProfile {
    dob: Option<chrono::NaiveDate>,
    name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
}

#[derive(Deserialize)]
pub struct GetProfilePhoto {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct VerificationQuery {
    pub id: i32,
    pub token: u64,
}

#[derive(Deserialize)]
pub struct VerificationClaims {
    pub id: i32,
    pub pass_hash: String,
}

static VERIFICATION_SEED: Lazy<u64> = Lazy::new(|| rand::random());

impl Into<u64> for VerificationClaims {
    fn into(self) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(*VERIFICATION_SEED);
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl std::hash::Hash for VerificationClaims {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_i32(self.id);
        state.write(self.pass_hash.as_bytes())
    }
}
