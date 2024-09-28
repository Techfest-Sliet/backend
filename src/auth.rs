use std::time::{Duration, SystemTime, SystemTimeError};

use argon2::{
    password_hash::{
        self, rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use axum::{extract::State, Form};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use diesel::prelude::*;
use http::StatusCode;
use jsonwebtoken::{DecodingKey, EncodingKey, Header};
use once_cell::sync::Lazy;
use rust_gmail::GmailClient;
use serde::{Deserialize, Serialize};

use crate::{
    forms::{faculty::FacultySignUp, student::StudentSignUp, users::SignInForm},
    models::users::User,
    schema::{faculty, students, users},
    state::SiteState,
};

pub async fn sign_in(
    State(state): State<SiteState>,
    cookie_jar: CookieJar,
    Form(data): Form<SignInForm>,
) -> Result<CookieJar, StatusCode> {
    let argon2 = Argon2::default();
    let user = users::table
        .select(User::as_select())
        .filter(users::email.eq(data.email))
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::UNAUTHORIZED
        })?;
    let pass = PasswordHash::new(&user.password_hash).map_err(|e| {
        log::error!("{e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    match argon2.verify_password(data.password.as_bytes(), &pass) {
        Ok(_) => {
            let claims: UserClaims = (&user).try_into().map_err(|e| {
                log::error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            let cookie: Cookie = (&claims).try_into().map_err(|e| {
                log::error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            Ok(cookie_jar.add(cookie))
        }
        Err(e) => {
            log::error!("{e:?}");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

pub async fn student_sign_up(
    State(state): State<SiteState>,
    cookie_jar: CookieJar,
    Form(data): Form<StudentSignUp>,
) -> Result<CookieJar, StatusCode> {
    if let Some((_, email_domain)) = data.email.rsplit_once('@') {
        if email_domain.to_lowercase() != "sliet.ac.in" {
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }
    let user: User = data.clone().try_into().map_err(|v| {
        log::error!("{v:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    user.send_verification_email(&state.mailer)
        .await
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let user: User = user
        .insert_into(users::table)
        .returning(User::as_returning())
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::CONFLICT
        })?;
    data.to_student(&user)
        .insert_into(students::table)
        .execute(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::CONFLICT
        })?;
    let claims: UserClaims = (&user).try_into().map_err(|e| {
        log::error!("{e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let cookie: Cookie = (&claims).try_into().map_err(|e| {
        log::error!("{e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(cookie_jar.add(cookie))
}

pub async fn faculty_sign_up(
    State(state): State<SiteState>,
    cookie_jar: CookieJar,
    Form(data): Form<FacultySignUp>,
) -> Result<CookieJar, StatusCode> {
    let user: User = data.clone().try_into().map_err(|v| {
        log::error!("{v:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    user.send_verification_email(&state.mailer)
        .await
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let user: User = user
        .insert_into(users::table)
        .returning(User::as_returning())
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::CONFLICT
        })?;
    data.to_faculty(&user)
        .insert_into(faculty::table)
        .execute(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::CONFLICT
        })?;
    let claims: UserClaims = (&user).try_into().map_err(|e| {
        log::error!("{e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let cookie: Cookie = (&claims).try_into().map_err(|e| {
        log::error!("{e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(cookie_jar.add(cookie))
}

#[derive(Serialize, Deserialize)]
pub struct UserClaims {
    pub id: i32,
    pub hash: String,
    pub exp: u64,
}

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

// Keys for encoding JWTs
impl Keys {
    fn new(secret: &[u8]) -> Keys {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET Environment Vaiable not set, it must be set.");
    Keys::new(secret.as_bytes())
});

impl TryFrom<&User> for UserClaims {
    type Error = SystemTimeError;

    fn try_from(value: &User) -> Result<Self, Self::Error> {
        Ok(UserClaims {
            id: value.id,
            hash: value.password_hash.clone(),
            exp: (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?
                + Duration::from_secs(60 * 60 * 24))
            .as_secs(),
        })
    }
}

impl<'a: 'static> TryInto<Cookie<'a>> for &UserClaims {
    type Error = jsonwebtoken::errors::Error;

    fn try_into(self) -> Result<Cookie<'a>, Self::Error> {
        jsonwebtoken::encode(&Header::default(), self, &KEYS.encoding).map(|s| {
            Cookie::build(("jwt-token", s))
                .http_only(true)
                .secure(true)
                .same_site(SameSite::None)
                .max_age(Duration::from_secs(60 * 60 * 12).try_into().unwrap())
                .partitioned(true)
                .path("/")
                .build()
        })
    }
}
