use std::io::Cursor;

use axum::{
    body::{Body, Bytes},
    extract::{Query, State},
    response::IntoResponse,
    Form, Json,
};
use axum_macros::debug_handler;
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use diesel::prelude::*;
use highway::HighwayHash;
use http::{header, HeaderMap, StatusCode};
use tokio_util::io::ReaderStream;

use crate::{
    forms::{
        users::Profile,
        workshops::{
            AddWorkshopStudentCoordinator, ChangeWorkshop, CreateWorkshop, DeleteWorkshop,
            GetWorkshopStudentCoordinator, WorkshopId, WorkshopIndividualAttendance,
        },
    },
    models::{
        students::{Student, StudentResponse},
        users::{Role, User},
        workshops::Workshop,
    },
    schema::{
        domains, faculty_coordinators, student_domain_coordinators, student_workshop_coordinators,
        students, users, workshop_participation, workshops,
    },
    state::SiteState,
};

pub async fn get_workshop(
    State(state): State<SiteState>,
) -> Result<Json<Vec<Workshop>>, StatusCode> {
    workshops::table
        .select(Workshop::as_select())
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

pub async fn create_workshop(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<CreateWorkshop>,
) -> Result<Json<Workshop>, StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    log::info!("{:?}", user);
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = faculty_coordinators::table
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(data.domain_id))
                .filter(faculty_coordinators::faculty_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                })?;
        }
        Role::STUDENT_COORDINATOR => {
            let _: i32 = workshops::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(workshops::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                })?;
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    }
    data.insert_into(workshops::table)
        .returning(Workshop::as_returning())
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
pub async fn delete_workshop(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<DeleteWorkshop>,
) -> Result<Json<Workshop>, StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = workshops::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(workshops::domain_id))
                .filter(faculty_coordinators::faculty_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                })?;
        }
        Role::STUDENT_COORDINATOR => {
            let _: i32 = workshops::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(workshops::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                })?;
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    }
    diesel::delete(workshops::table)
        .filter(workshops::id.eq(data.id))
        .returning(Workshop::as_returning())
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

pub async fn change_workshop(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<ChangeWorkshop>,
) -> Result<Json<Workshop>, StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = workshops::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(workshops::domain_id))
                .filter(faculty_coordinators::faculty_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                })?;
        }
        Role::STUDENT_COORDINATOR => {
            let student_domain_id: Result<i32, StatusCode> = workshops::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(workshops::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            let student_workshop_id: Result<i32, StatusCode> = workshops::table
                .inner_join(student_workshop_coordinators::table)
                .select(student_workshop_coordinators::workshop_id)
                .filter(student_workshop_coordinators::workshop_id.eq(data.id))
                .filter(student_workshop_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            match student_domain_id {
                Ok(_) => {}
                Err(_) => match student_workshop_id {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e);
                    }
                },
            }
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    }
    diesel::update(workshops::table)
        .filter(workshops::id.eq(data.id))
        .set(data)
        .returning(Workshop::as_returning())
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
pub async fn set_workshop_photo(
    State(state): State<SiteState>,
    user: User,
    Query(data): Query<WorkshopId>,
    photo: Bytes,
) -> Result<(), StatusCode> {
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = workshops::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(workshops::domain_id))
                .filter(faculty_coordinators::faculty_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                })?;
        }
        Role::STUDENT_COORDINATOR => {
            let student_domain_id: Result<i32, StatusCode> = workshops::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(workshops::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            let student_workshop_id: Result<i32, StatusCode> = workshops::table
                .inner_join(student_workshop_coordinators::table)
                .select(student_workshop_coordinators::workshop_id)
                .filter(student_workshop_coordinators::workshop_id.eq(data.id))
                .filter(student_workshop_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            match student_domain_id {
                Ok(_) => {}
                Err(_) => match student_workshop_id {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e);
                    }
                },
            }
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
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
    diesel::update(workshops::table)
        .filter(workshops::id.eq(user.id))
        .set(
            workshops::photo_hash.eq(hash
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
pub async fn get_workshop_photo(
    State(state): State<SiteState>,
    Query(data): Query<WorkshopId>,
) -> impl IntoResponse {
    // `File` implements `AsyncRead`
    let photo_hash = workshops::table
        .select(workshops::photo_hash)
        .filter(workshops::id.eq(data.id))
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

pub async fn get_workshop_coordinator(
    State(state): State<SiteState>,
    Query(data): Query<GetWorkshopStudentCoordinator>,
) -> Result<Json<Vec<StudentResponse>>, StatusCode> {
    student_workshop_coordinators::table
        .inner_join(workshops::table)
        .inner_join(students::table.inner_join(users::table))
        .filter(workshops::id.eq(data.id))
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

pub async fn add_workshop_coordinator(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<AddWorkshopStudentCoordinator>,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = workshops::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(workshops::domain_id))
                .filter(faculty_coordinators::faculty_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                })?;
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    }
    data.insert_into(student_workshop_coordinators::table)
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

pub async fn get_workshop_attendance(
    State(state): State<SiteState>,
    user: User,
    Query(data): Query<WorkshopId>,
) -> Result<Json<Vec<i32>>, StatusCode> {
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = workshops::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(workshops::domain_id))
                .filter(faculty_coordinators::faculty_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                })?;
        }
        Role::STUDENT_COORDINATOR => {
            let student_domain_id: Result<i32, StatusCode> = workshops::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(workshops::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            let student_workshop_id: Result<i32, StatusCode> = workshops::table
                .inner_join(student_workshop_coordinators::table)
                .select(student_workshop_coordinators::workshop_id)
                .filter(student_workshop_coordinators::workshop_id.eq(data.id))
                .filter(student_workshop_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            match student_domain_id {
                Ok(_) => {}
                Err(_) => match student_workshop_id {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e);
                    }
                },
            }
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    workshop_participation::table
        .select(workshop_participation::user_id)
        .filter(workshop_participation::workshop_id.eq(data.id))
        .load::<i32>(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map(|v| Json(v))
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn mark_workshop_attendance(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<WorkshopIndividualAttendance>,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = workshops::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(workshops::domain_id))
                .filter(faculty_coordinators::faculty_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                })?;
        }
        Role::STUDENT_COORDINATOR => {
            let student_domain_id: Result<i32, StatusCode> = workshops::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(workshops::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            let student_workshop_id: Result<i32, StatusCode> = workshops::table
                .inner_join(student_workshop_coordinators::table)
                .select(student_workshop_coordinators::workshop_id)
                .filter(student_workshop_coordinators::workshop_id.eq(data.workshop_id))
                .filter(student_workshop_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            match student_domain_id {
                Ok(_) => {}
                Err(_) => match student_workshop_id {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e);
                    }
                },
            }
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    diesel::update(workshop_participation::table)
        .set(workshop_participation::attended.eq(true))
        .filter(workshop_participation::user_id.eq(data.user_id))
        .filter(workshop_participation::workshop_id.eq(data.workshop_id))
        .execute(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(())
}

pub async fn remove_workshop_individual_attendance(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<WorkshopIndividualAttendance>,
) -> Result<(), StatusCode> {
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = workshops::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(workshops::domain_id))
                .filter(faculty_coordinators::faculty_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                })?;
        }
        Role::STUDENT_COORDINATOR => {
            let student_domain_id: Result<i32, StatusCode> = workshops::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(workshops::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            let student_workshop_id: Result<i32, StatusCode> = workshops::table
                .inner_join(student_workshop_coordinators::table)
                .select(student_workshop_coordinators::workshop_id)
                .filter(student_workshop_coordinators::workshop_id.eq(data.workshop_id))
                .filter(student_workshop_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            match student_domain_id {
                Ok(_) => {}
                Err(_) => match student_workshop_id {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e);
                    }
                },
            }
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    diesel::update(workshop_participation::table)
        .set(workshop_participation::attended.eq(false))
        .filter(workshop_participation::user_id.eq(data.user_id))
        .filter(workshop_participation::workshop_id.eq(data.workshop_id))
        .returning(workshop_participation::user_id)
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

pub async fn leave_workshop_individual(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<WorkshopId>,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    diesel::delete(workshop_participation::table)
        .filter(workshop_participation::user_id.eq(user.id))
        .filter(workshop_participation::workshop_id.eq(data.id))
        .execute(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::NOT_MODIFIED
        })
        .map(|_| ())
}

pub async fn join_workshop(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<WorkshopId>,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    WorkshopIndividualAttendance {
        user_id: user.id,
        workshop_id: data.id,
    }
    .insert_into(workshop_participation::table)
    .execute(&mut state.connection.get().map_err(|e| {
        log::error!("{e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?)
    .map_err(|e| {
        log::error!("{e:?}");
        StatusCode::NOT_MODIFIED
    })
    .map(|_| ())
}


pub async fn joined_workshops_individual(
    State(state): State<SiteState>,
    user: User,
) -> Result<Json<Vec<Workshop>>, StatusCode> {
    workshop_participation::table
        .inner_join(workshops::table)
        .select(Workshop::as_select())
        .filter(workshop_participation::user_id.eq(user.id))
        .load(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::NOT_MODIFIED
        })
        .map(|v| Json(v))
}
