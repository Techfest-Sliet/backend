use argon2::{
    password_hash::{self, rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use serde::Deserialize;

use crate::models::{
    students::{Department, Student},
    users::{Role, User},
};

#[derive(Deserialize, Clone)]
pub struct StudentSignUp {
    pub name: String,
    pub dob: chrono::NaiveDate,
    pub email: String,
    pub phone: String,
    pub password: String,
    pub college: String,
    pub reg_no: String,
    pub dept: Department,
}

impl TryInto<User> for StudentSignUp {
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
            role: Role::PARTICIPANT,
            photo_hash: None,
            verified: false,
            password_hash: password_hash.to_string(),
        })
    }
}

impl StudentSignUp {
    pub fn to_student(self, user: &User) -> Student {
        Student {
            user_id: user.id,
            college: self.college,
            reg_no: self.reg_no,
            dept: self.dept,
        }
    }
}
