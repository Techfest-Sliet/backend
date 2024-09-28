use argon2::{
    password_hash::{
        self, rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use serde::Deserialize;

use crate::models::{
    faculty::{Faculty, Title}, students::Department, users::{Role, User}
};

#[derive(Deserialize, Clone)]
pub struct FacultySignUp {
    pub name: String,
    pub dob: chrono::NaiveDate,
    pub email: String,
    pub phone: String,
    pub role: Role,
    pub password: String,
    pub title: Title,
    pub dept: Department,
}

impl TryInto<User> for FacultySignUp {
    type Error = password_hash::Error;

    fn try_into(self) -> Result<User, Self::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(self.password.as_bytes(), &salt)?;

        Ok(User {
            id: 0,
            name: self.name,
            dob: self.dob,
            email: self.email,
            phone: self.phone,
            role: self.role,
            photo_hash: None,
            verified: false,
            password_hash: password_hash.to_string(),
        })
    }
}

impl FacultySignUp {
    pub fn to_faculty(self, user: &User) -> Faculty {
        Faculty {
            user_id: user.id,
            title: self.title,
            dept: self.dept,
        }
    }
}
