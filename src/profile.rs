use argon2::password_hash::SaltString;
use argon2::Argon2;
use argon2::PasswordHasher;
use base64::prelude::*;
use rand::rngs::OsRng;
use std::collections::HashMap;
use std::io::Cursor;

use axum::body::{Body, Bytes};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::{Form, Json};
use axum_macros::debug_handler;
use diesel::prelude::*;
use highway::HighwayHash;
use http::{header, HeaderMap, StatusCode};
use tokio_util::io::ReaderStream;

use crate::forms::faculty::NewFacultyProfile;
use crate::forms::student::NewStudentProfile;
use crate::forms::users::{
    ChangeProfile, GetProfilePhoto, PasswordResetQuery, Profile, ResetClaims, ResetSendQuery,
    VerificationClaims, VerificationQuery,
};
use crate::models::faculty::Faculty;
use crate::models::students::{Department, Student};
use crate::models::team::TeamRequest;
use crate::models::users::User;
use crate::schema::{faculty, students, team_requests, users};
use crate::state::SiteState;

pub async fn get_profile(
    State(state): State<SiteState>,
    user: User,
) -> Result<Json<Profile>, StatusCode> {
    users::table
        .select(Profile::as_select())
        .filter(users::id.eq(user.id))
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .map(|v| Json(v))
}

pub async fn change_profile(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<ChangeProfile>,
) -> Result<Json<Profile>, StatusCode> {
    diesel::update(users::table)
        .set(data)
        .filter(users::id.eq(user.id))
        .returning(Profile::as_returning())
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .map(|v| Json(v))
}

// Not Deleting the image in case some other user also happens to have the same exact image
pub async fn set_profile_photo(
    State(state): State<SiteState>,
    user: User,
    photo: Bytes,
) -> Result<(), StatusCode> {
    let hash = state.bulk_hasher.hash256(photo.to_vec().as_slice());
    let photo = image::ImageReader::new(Cursor::new(photo))
        .with_guessed_format()
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::BAD_REQUEST
        })?
        .decode()
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::BAD_REQUEST
        })?;
    let photo = photo.thumbnail(512, 512);
    photo
        .save_with_format(
            format!(
                "{}/{}",
                state.image_dir.to_string_lossy(),
                BASE64_URL_SAFE_NO_PAD.encode(
                    hash.map(|v| v.to_le_bytes())
                        .into_iter()
                        .flatten()
                        .collect::<Vec<u8>>()
                )
            ),
            image::ImageFormat::Avif,
        )
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::BAD_REQUEST
        })?;
    diesel::update(users::table)
        .filter(users::id.eq(user.id))
        .set(
            users::photo_hash.eq(hash
                .map(|v| v.to_le_bytes())
                .into_iter()
                .flatten()
                .collect::<Vec<u8>>()),
        )
        .execute(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::BAD_REQUEST
        })?;
    Ok(())
}

#[debug_handler]
pub async fn get_profile_photo(
    State(state): State<SiteState>,
    user: User,
    query: Option<Query<GetProfilePhoto>>,
) -> impl IntoResponse {
    // `File` implements `AsyncRead`
    let photo_hash = if let Some(other_profile) = query {
        users::table
            .select(users::photo_hash)
            .filter(users::id.eq(other_profile.id))
            .get_result(&mut state.connection.get().map_err(|e| {
                log::error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?)
            .map_err(|e| {
                log::error!("{e:?}");
                StatusCode::NOT_FOUND
            })?
    } else {
        user.photo_hash
    };
    let photo_hash = match photo_hash {
        None => return Err(StatusCode::NOT_FOUND),
        Some(v) => v,
    };
    let file = match tokio::fs::File::open(format!(
        "{}/{}",
        state.image_dir.to_string_lossy(),
        BASE64_URL_SAFE_NO_PAD.encode(photo_hash)
    ))
    .await
    {
        Ok(file) => file,
        Err(e) => {
            log::error!("{e:?}");
            return Err(StatusCode::NOT_FOUND);
        }
    };
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut header_map = HeaderMap::new();
    header_map.insert(
        header::CONTENT_TYPE,
        "image/avif"
            .parse()
            .expect("Parsing \"image/avif\" should have been fine."),
    );

    Ok((header_map, body))
}

pub async fn get_individual_team_requests(
    State(state): State<SiteState>,
    user: User,
) -> Result<Json<Vec<TeamRequest>>, StatusCode> {
    team_requests::table
        .select(TeamRequest::as_select())
        .filter(team_requests::student_id.eq(user.id))
        .load(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map(|v| Json(v))
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::UNAUTHORIZED
        })
}

pub async fn send_reset_mail(
    State(state): State<SiteState>,
    Form(data): Form<ResetSendQuery>,
) -> Result<(), StatusCode> {
    let user = users::table
        .select(User::as_select())
        .filter(users::email.eq(data.email.trim()))
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::UNAUTHORIZED
        })?;
    user.send_password_reset_email(state.mailer, &state.mail_builder)
        .await
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn reset_password(
    State(state): State<SiteState>,
    Form(data): Form<PasswordResetQuery>,
) -> Result<(), StatusCode> {
    let user: User = users::table
        .select(User::as_select())
        .filter(users::id.eq(data.id))
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::UNAUTHORIZED
        })?;
    let valid_verification_claims: u64 = ResetClaims::from(&user).into();
    if data.token != valid_verification_claims {
        log::error!("Password reset tokens do not match");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(data.password.as_bytes(), &salt)
        .map(|v| v.to_string())
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    diesel::update(users::table)
        .set(users::password_hash.eq(password_hash))
        .filter(users::id.eq(user.id))
        .execute(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map(|_| ())
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn verify_user(
    State(state): State<SiteState>,
    Query(data): Query<VerificationQuery>,
) -> Result<Redirect, StatusCode> {
    let pass_hash: String = users::table
        .select(users::password_hash)
        .filter(users::id.eq(data.id))
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::UNAUTHORIZED
        })?;
    let verification_claims: u64 = VerificationClaims {
        id: data.id,
        pass_hash,
    }
    .into();
    if verification_claims == data.token {
        diesel::update(users::table)
            .filter(users::id.eq(data.id))
            .set(users::verified.eq(true))
            .execute(&mut state.connection.get().map_err(|e| {
                log::error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?)
            .map(|_| Redirect::to("https://techfestsliet.org/"))
            .map_err(|e| {
                log::error!("{e:?}");
                StatusCode::UNAUTHORIZED
            })
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn get_departments() -> Json<HashMap<String, String>> {
    Json(HashMap::<String, String>::from_iter(
        Department::VARIANTS
            .iter()
            .map(|v| (format!("{:?}", v), format!("{}", v))),
    ))
}

pub async fn get_student_profile(
    State(state): State<SiteState>,
    user: User,
) -> Result<Json<Student>, StatusCode> {
    students::table
        .select(Student::as_select())
        .filter(students::user_id.eq(user.id))
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .map(|v| Json(v))
}

pub async fn get_faculty_profile(
    State(state): State<SiteState>,
    user: User,
) -> Result<Json<Faculty>, StatusCode> {
    faculty::table
        .select(Faculty::as_select())
        .filter(faculty::user_id.eq(user.id))
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .map(|v| Json(v))
}

pub async fn create_student_profile(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<NewStudentProfile>,
) -> Result<Json<Student>, StatusCode> {
    Student {
        user_id: user.id,
        reg_no: data.reg_no,
        college: data.college,
        dept: data.dept,
    }
    .insert_into(students::table)
    .returning(Student::as_returning())
    .get_result(&mut state.connection.get().map_err(|e| {
        log::error!("{e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?)
    .map_err(|e| {
        log::error!("{e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
    .map(|v| Json(v))
}

#[debug_handler]
pub async fn create_faculty_profile(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<NewFacultyProfile>,
) -> Result<Json<Faculty>, StatusCode> {
    Faculty {
        user_id: user.id,
        title: data.title,
        dept: data.dept,
    }
    .insert_into(faculty::table)
    .returning(Faculty::as_returning())
    .get_result(&mut state.connection.get().map_err(|e| {
        log::error!("{e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?)
    .map_err(|e| {
        log::error!("{e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
    .map(|v| Json(v))
}
