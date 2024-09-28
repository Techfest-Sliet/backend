use std::io::Cursor;

use axum::{
    body::{Body, Bytes},
    extract::{Query, State},
    response::IntoResponse,
    Form, Json,
};
use axum_macros::debug_handler;
use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use diesel::prelude::*;
use highway::HighwayHash;
use http::{header, HeaderMap, StatusCode};
use tokio_util::io::ReaderStream;

use crate::{
    forms::{
        domains::{
            AddDomainFacultyCoordinator, AddDomainStudentCoordinator, ChangeDomain, CreateDomain,
            DeleteDomain, GetDomainFacultyCoordinator, GetDomainPhoto,
        },
        users::Profile,
    },
    models::{
        domains::Domain,
        faculty::{Faculty, FacultyResponse},
        students::{Student, StudentResponse},
        users::{Role, User},
    },
    schema::{
        domains, faculty, faculty_coordinators, student_domain_coordinators, students, users,
    },
    state::SiteState,
};

pub async fn get_domain(State(state): State<SiteState>) -> Result<Json<Vec<Domain>>, StatusCode> {
    domains::table
        .select(Domain::as_select())
        .get_results(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map(|v| Json(v))
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn create_domain(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<CreateDomain>,
) -> Result<Json<Domain>, StatusCode> {
    match user.role {
        Role::SUPER_ADMIN => {}
        _ => return Err(StatusCode::UNAUTHORIZED),
    }
    data.insert_into(domains::table)
        .returning(Domain::as_returning())
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map(|v| Json(v))
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[debug_handler]
pub async fn delete_domain(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<DeleteDomain>,
) -> Result<Json<Domain>, StatusCode> {
    match user.role {
        Role::SUPER_ADMIN => {}
        _ => return Err(StatusCode::UNAUTHORIZED),
    }
    diesel::delete(domains::table)
        .filter(domains::id.eq(data.id))
        .returning(Domain::as_returning())
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map(|v| Json(v))
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn change_domain(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<ChangeDomain>,
) -> Result<Json<Domain>, StatusCode> {
    match user.role {
        Role::SUPER_ADMIN => {}
        _ => return Err(StatusCode::UNAUTHORIZED),
    }
    diesel::update(domains::table)
        .filter(domains::id.eq(data.id))
        .set(data)
        .returning(Domain::as_returning())
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map(|v| Json(v))
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

// Not Deleting the image in case some other user also happens to have the same exact image
pub async fn set_domain_photo(
    State(state): State<SiteState>,
    user: User,
    photo: Bytes,
) -> Result<(), StatusCode> {
    match user.role {
        Role::SUPER_ADMIN => {}
        _ => return Err(StatusCode::UNAUTHORIZED),
    }
    if !user.verified {
        return Err(StatusCode::UNAUTHORIZED);
    }
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
                BASE64_STANDARD_NO_PAD.encode(
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
    diesel::update(domains::table)
        .filter(domains::id.eq(user.id))
        .set(
            domains::photo_hash.eq(hash
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
pub async fn get_domain_photo(
    State(state): State<SiteState>,
    Query(data): Query<GetDomainPhoto>,
) -> impl IntoResponse {
    // `File` implements `AsyncRead`
    let photo_hash = domains::table
        .select(domains::photo_hash)
        .filter(domains::id.eq(data.id))
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::NOT_FOUND
        })?;
    let photo_hash: Vec<u8> = match photo_hash {
        None => return Err(StatusCode::NOT_FOUND),
        Some(v) => v,
    };
    let file = match tokio::fs::File::open(format!(
        "{}/{}",
        state.image_dir.to_string_lossy(),
        BASE64_STANDARD_NO_PAD.encode(photo_hash)
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

pub async fn get_domain_faculty_coordinator(
    State(state): State<SiteState>,
    Query(data): Query<GetDomainFacultyCoordinator>,
) -> Result<Json<Vec<FacultyResponse>>, StatusCode> {
    faculty_coordinators::table
        .inner_join(domains::table)
        .inner_join(faculty::table.inner_join(users::table))
        .filter(domains::id.eq(data.id))
        .select((Faculty::as_select(), Profile::as_select()))
        .get_results(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::NOT_FOUND
        })?)
        .map(|v: Vec<(Faculty, Profile)>| {
            Json(
                v.into_iter()
                    .map(|(faculty, profile)| FacultyResponse { faculty, profile })
                    .collect::<Vec<FacultyResponse>>(),
            )
        })
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::NOT_FOUND
        })
}

pub async fn add_domain_faculty_coordinator(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<AddDomainFacultyCoordinator>,
) -> StatusCode {
    match user.role {
        Role::SUPER_ADMIN => {}
        _ => return StatusCode::UNAUTHORIZED,
    }
    if !user.verified {
        return StatusCode::UNAUTHORIZED;
    }
    match data
        .insert_into(faculty_coordinators::table)
        .execute(&mut match state.connection.get() {
            Ok(v) => v,
            Err(e) => {
                log::error!("{e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            log::error!("{e:?}");
            StatusCode::BAD_REQUEST
        }
    }
}

pub async fn get_domain_student_coordinator(
    State(state): State<SiteState>,
    Query(data): Query<GetDomainFacultyCoordinator>,
) -> Result<Json<Vec<StudentResponse>>, StatusCode> {
    student_domain_coordinators::table
        .inner_join(domains::table)
        .inner_join(students::table.inner_join(users::table))
        .filter(domains::id.eq(data.id))
        .select((Student::as_select(), Profile::as_select()))
        .get_results(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::NOT_FOUND
        })?)
        .map(|v: Vec<(Student, Profile)>| {
            Json(
                v.into_iter()
                    .map(|(student, profile)| StudentResponse { student, profile })
                    .collect::<Vec<StudentResponse>>(),
            )
        })
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::NOT_FOUND
        })
}

pub async fn add_domain_student_coordinator(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<AddDomainStudentCoordinator>,
) -> StatusCode {
    match user.role {
        Role::SUPER_ADMIN => {}
        _ => return StatusCode::UNAUTHORIZED,
    }
    if !user.verified {
        return StatusCode::UNAUTHORIZED;
    }
    match data
        .insert_into(student_domain_coordinators::table)
        .execute(&mut match state.connection.get() {
            Ok(v) => v,
            Err(e) => {
                log::error!("{e:?}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }) {
        Err(e) => {
            log::error!("{e:?}");
            return StatusCode::BAD_REQUEST;
        }
        Ok(_) => StatusCode::OK,
    }
}
