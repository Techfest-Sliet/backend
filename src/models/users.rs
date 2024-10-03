use std::{fmt::Display, sync::Arc};

use axum::{async_trait, extract::FromRequestParts};
use axum_extra::extract::cookie::Cookie;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, ManageConnection, Pool},
};
use http::{request::Parts, StatusCode};
use jsonwebtoken::Validation;
use mail_send::{mail_builder::MessageBuilder, SmtpClient};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    auth::{UserClaims, KEYS},
    forms::users::VerificationClaims,
    schema::{payments, users},
    state::SiteState,
};
#[derive(diesel_derive_enum::DbEnum, Debug, Clone, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::Role"]
#[allow(non_camel_case_types)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum Role {
    SUPER_ADMIN,
    FACULTY_COORDINATOR,
    STUDENT_COORDINATOR,
    PARTICIPANT,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::SUPER_ADMIN => "Super Admin",
            Self::FACULTY_COORDINATOR => "Faculty Coordinator",
            Self::STUDENT_COORDINATOR => "Student Coordinator",
            Self::PARTICIPANT => "Participant",
        })
    }
}

impl Role {
    pub const VARIANTS: [Role; 4] = [
        Role::SUPER_ADMIN,
        Role::FACULTY_COORDINATOR,
        Role::STUDENT_COORDINATOR,
        Role::PARTICIPANT,
    ];
}

#[derive(Queryable, Selectable, Insertable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub name: String,
    pub dob: chrono::NaiveDate,
    pub email: String,
    pub phone: String,
    pub role: Role,
    pub photo_hash: Option<Vec<u8>>,
    pub verified: bool,
    pub password_hash: String,
}

static EMAIL_TEMPLATE: &'static str = include_str!("verification_email.html");

impl User {
    pub fn is_payment_done(&self, db: &Pool<ConnectionManager<PgConnection>>) -> bool {
        if let Some((_, "sliet.ac.in")) = self.email.trim_ascii().rsplit_once('@') {
            true
        } else {
            match payments::table
                .select(payments::verified)
                .filter(payments::user_id.eq(self.id))
                .filter(payments::verified.eq(true))
                .get_result(&mut match db.get() {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("{e:?}");
                        return false;
                    }
                }) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("{e:?}");
                    return false;
                }
            }
        }
    }
    pub async fn send_verification_email<
        T: Unpin + tokio::io::AsyncRead + tokio::io::AsyncWrite,
    >(
        &self,
        mailer: Arc<Mutex<SmtpClient<T>>>,
    ) -> mail_send::Result<()> {
        let verification_claims: u64 = VerificationClaims {
            id: self.id,
            pass_hash: self.password_hash.clone(),
        }
        .into();
        let replace = EMAIL_TEMPLATE.replace(
            "{verification_query}",
            &format!("?id={}&token={}", self.id, verification_claims),
        );
        let message = MessageBuilder::new()
            .from(("Techfest", "techfest@sliet.ac.in"))
            .to((self.name.clone(), self.email.clone()))
            .subject("Email verification for techfest 24")
            .html_body(&replace);
        let display_message = message.clone().write_to_string().unwrap();
        log::info!("Mail being sent is: {}", display_message);
        mailer.lock().await.send(message).await
    }
}

#[async_trait]
impl<'a> FromRequestParts<SiteState> for User {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &SiteState,
    ) -> Result<Self, Self::Rejection> {
        match parts.headers.get("Cookie") {
            Some(cookie_string) => match Cookie::split_parse(match cookie_string.to_str() {
                Ok(s) => s,
                Err(_) => return Err((StatusCode::BAD_REQUEST, "Invalid Cookies".to_string())),
            })
            .flatten()
            .filter(|c| c.name() == "jwt-token")
            .next()
            {
                Some(c) => match jsonwebtoken::decode::<UserClaims>(
                    c.value(),
                    &KEYS.decoding,
                    &Validation::new(jsonwebtoken::Algorithm::HS256),
                ) {
                    Ok(token) => {
                        let user =
                            match users::table
                                .filter(users::id.eq(token.claims.id))
                                .select(User::as_select())
                                .get_result(&mut state.connection.get().map_err(|e| {
                                    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                                })?) {
                                Ok(user) => user,
                                Err(e) => {
                                    log::error!("{}", e);
                                    return Err((
                                        StatusCode::UNAUTHORIZED,
                                        "Incorrent Username or Password".to_string(),
                                    ));
                                }
                            };

                        if user.password_hash == token.claims.hash {
                            return Ok(user);
                        }
                        return Err((
                            StatusCode::UNAUTHORIZED,
                            "Incorrect username or password".to_string(),
                        ));
                    }
                    Err(e) => {
                        log::error!("At line {}, {}", line!(), e);
                        return Err((
                            StatusCode::BAD_REQUEST,
                            "Could not parse the JWT".to_string(),
                        ));
                    }
                },
                None => {
                    return Err((StatusCode::UNAUTHORIZED, "JWT Cookie not found".to_string()));
                }
            },
            None => {
                return Err((StatusCode::UNAUTHORIZED, "JWT Cookie not found".to_string())).into()
            }
        };
    }
}
