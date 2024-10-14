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
        domains::GetDomainEvent,
        events::{
            AddEventStudentCoordinator, ChangeEvent, CreateEvent, DeleteEvent, EventId,
            EventIndividualAttendance, EventTeamAttendance, GetEventStudentCoordinator,
        },
        teams::TeamId,
        users::Profile,
    },
    models::{
        domains::Domain, events::Event, students::{Student, StudentResponse}, users::{Role, User}
    },
    schema::{
        domains, events, faculty_coordinators, individual_event_participation,
        student_domain_coordinators, student_event_coordinators, students,
        team_event_participations, team_members, teams, users,
    },
    state::SiteState,
};

pub async fn get_event(
    State(state): State<SiteState>,
    Query(data): Query<EventId>,
) -> Result<Json<Event>, StatusCode> {
    events::table
        .select(Event::as_select())
        .filter(events::id.eq(data.id))
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

pub async fn get_events_by_domain(
    State(state): State<SiteState>,
    Query(data): Query<GetDomainEvent>,
) -> Result<Json<Vec<Event>>, StatusCode> {
    events::table
        .select(Event::as_select())
        .filter(events::domain_id.eq(data.id))
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

pub async fn create_event(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<CreateEvent>,
) -> Result<Json<Event>, StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
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
            let _: i32 = events::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(events::domain_id))
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
    data.insert_into(events::table)
        .returning(Event::as_returning())
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
pub async fn delete_event(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<DeleteEvent>,
) -> Result<Json<Event>, StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = events::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(events::domain_id))
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
            let _: i32 = events::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(events::domain_id))
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
    diesel::delete(events::table)
        .filter(events::id.eq(data.id))
        .returning(Event::as_returning())
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

pub async fn change_event(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<ChangeEvent>,
) -> Result<Json<Event>, StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = events::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(events::domain_id))
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
            let student_domain_id: Result<i32, StatusCode> = events::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(events::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            let student_event_id: Result<i32, StatusCode> = events::table
                .inner_join(student_event_coordinators::table)
                .select(student_event_coordinators::event_id)
                .filter(student_event_coordinators::event_id.eq(data.id))
                .filter(student_event_coordinators::student_id.eq(user.id))
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
                Err(_) => match student_event_id {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e);
                    }
                },
            }
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    }
    diesel::update(events::table)
        .filter(events::id.eq(data.id))
        .set(data)
        .returning(Event::as_returning())
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
pub async fn set_event_photo(
    State(state): State<SiteState>,
    user: User,
    Query(data): Query<EventId>,
    photo: Bytes,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = events::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(events::domain_id))
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
            let student_domain_id: Result<i32, StatusCode> = events::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(events::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            let student_event_id: Result<i32, StatusCode> = events::table
                .inner_join(student_event_coordinators::table)
                .select(student_event_coordinators::event_id)
                .filter(student_event_coordinators::event_id.eq(data.id))
                .filter(student_event_coordinators::student_id.eq(user.id))
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
                Err(_) => match student_event_id {
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
    diesel::update(events::table)
        .filter(events::id.eq(user.id))
        .set(
            events::photo_hash.eq(hash
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
pub async fn get_event_photo(
    State(state): State<SiteState>,
    Query(data): Query<EventId>,
) -> impl IntoResponse {
    // `File` implements `AsyncRead`
    let photo_hash = events::table
        .select(events::photo_hash)
        .filter(events::id.eq(data.id))
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

pub async fn get_event_coordinator(
    State(state): State<SiteState>,
    Query(data): Query<GetEventStudentCoordinator>,
) -> Result<Json<Vec<StudentResponse>>, StatusCode> {
    student_event_coordinators::table
        .inner_join(events::table)
        .inner_join(students::table.inner_join(users::table))
        .filter(events::id.eq(data.id))
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

pub async fn add_event_coordinator(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<AddEventStudentCoordinator>,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    match user.role {
        Role::SUPER_ADMIN => {}
        _ => return Err(StatusCode::UNAUTHORIZED),
    }
    data.insert_into(student_event_coordinators::table)
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

pub async fn get_event_individual_attendance(
    State(state): State<SiteState>,
    user: User,
    Query(data): Query<EventId>,
) -> Result<Json<Vec<i32>>, StatusCode> {
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = events::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(events::domain_id))
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
            let student_domain_id: Result<i32, StatusCode> = events::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(events::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            let student_event_id: Result<i32, StatusCode> = events::table
                .inner_join(student_event_coordinators::table)
                .select(student_event_coordinators::event_id)
                .filter(student_event_coordinators::event_id.eq(data.id))
                .filter(student_event_coordinators::student_id.eq(user.id))
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
                Err(_) => match student_event_id {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e);
                    }
                },
            }
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    individual_event_participation::table
        .select(individual_event_participation::user_id)
        .filter(individual_event_participation::event_id.eq(data.id))
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

pub async fn mark_event_individual_attendance(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<EventIndividualAttendance>,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = events::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(events::domain_id))
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
            let student_domain_id: Result<i32, StatusCode> = events::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(events::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            let student_event_id: Result<i32, StatusCode> = events::table
                .inner_join(student_event_coordinators::table)
                .select(student_event_coordinators::event_id)
                .filter(student_event_coordinators::event_id.eq(data.event_id))
                .filter(student_event_coordinators::student_id.eq(user.id))
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
                Err(_) => match student_event_id {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e);
                    }
                },
            }
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    diesel::update(individual_event_participation::table)
        .set(individual_event_participation::attended.eq(true))
        .filter(individual_event_participation::user_id.eq(data.user_id))
        .filter(individual_event_participation::event_id.eq(data.event_id))
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

pub async fn remove_event_individual_attendance(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<EventIndividualAttendance>,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = events::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(events::domain_id))
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
            let student_domain_id: Result<i32, StatusCode> = events::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(events::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            let student_event_id: Result<i32, StatusCode> = events::table
                .inner_join(student_event_coordinators::table)
                .select(student_event_coordinators::event_id)
                .filter(student_event_coordinators::event_id.eq(data.event_id))
                .filter(student_event_coordinators::student_id.eq(user.id))
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
                Err(_) => match student_event_id {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e);
                    }
                },
            }
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    diesel::update(individual_event_participation::table)
        .set(individual_event_participation::attended.eq(false))
        .filter(individual_event_participation::user_id.eq(data.user_id))
        .filter(individual_event_participation::event_id.eq(data.event_id))
        .returning(individual_event_participation::user_id)
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

pub async fn get_event_team_attendance(
    State(state): State<SiteState>,
    user: User,
    Query(data): Query<EventId>,
) -> Result<Json<Vec<i32>>, StatusCode> {
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = events::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(events::domain_id))
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
            let student_domain_id: Result<i32, StatusCode> = events::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(events::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            let student_event_id: Result<i32, StatusCode> = events::table
                .inner_join(student_event_coordinators::table)
                .select(student_event_coordinators::event_id)
                .filter(student_event_coordinators::event_id.eq(data.id))
                .filter(student_event_coordinators::student_id.eq(user.id))
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
                Err(_) => match student_event_id {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e);
                    }
                },
            }
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    team_event_participations::table
        .select(team_event_participations::team_id)
        .filter(team_event_participations::event_id.eq(data.id))
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

pub async fn mark_event_team_attendance(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<EventIndividualAttendance>,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = events::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(events::domain_id))
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
            let student_domain_id: Result<i32, StatusCode> = events::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(events::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            let student_event_id: Result<i32, StatusCode> = events::table
                .inner_join(student_event_coordinators::table)
                .select(student_event_coordinators::event_id)
                .filter(student_event_coordinators::event_id.eq(data.event_id))
                .filter(student_event_coordinators::student_id.eq(user.id))
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
                Err(_) => match student_event_id {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e);
                    }
                },
            }
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    diesel::update(team_event_participations::table)
        .set(team_event_participations::attended.eq(true))
        .filter(team_event_participations::team_id.eq(data.user_id))
        .filter(team_event_participations::event_id.eq(data.event_id))
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

pub async fn remove_event_team_attendance(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<EventIndividualAttendance>,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    match user.role {
        Role::SUPER_ADMIN => {}
        Role::FACULTY_COORDINATOR => {
            let _: i32 = events::table
                .inner_join(domains::table.inner_join(faculty_coordinators::table))
                .select(faculty_coordinators::domain_id)
                .filter(faculty_coordinators::domain_id.eq(events::domain_id))
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
            let student_domain_id: Result<i32, StatusCode> = events::table
                .inner_join(domains::table.inner_join(student_domain_coordinators::table))
                .select(student_domain_coordinators::domain_id)
                .filter(student_domain_coordinators::domain_id.eq(events::domain_id))
                .filter(student_domain_coordinators::student_id.eq(user.id))
                .get_result(&mut state.connection.get().map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
                .map_err(|e| {
                    log::error!("{e:?}");
                    StatusCode::UNAUTHORIZED
                });
            let student_event_id: Result<i32, StatusCode> = events::table
                .inner_join(student_event_coordinators::table)
                .select(student_event_coordinators::event_id)
                .filter(student_event_coordinators::event_id.eq(data.event_id))
                .filter(student_event_coordinators::student_id.eq(user.id))
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
                Err(_) => match student_event_id {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e);
                    }
                },
            }
        }
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    diesel::update(team_event_participations::table)
        .set(team_event_participations::attended.eq(false))
        .filter(team_event_participations::team_id.eq(data.user_id))
        .filter(team_event_participations::event_id.eq(data.event_id))
        .returning(team_event_participations::team_id)
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

pub async fn join_event_individual(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<EventId>,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    EventIndividualAttendance {
        user_id: user.id,
        event_id: data.id,
    }
    .insert_into(individual_event_participation::table)
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

pub async fn leave_event_individual(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<EventId>,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    diesel::delete(individual_event_participation::table)
        .filter(individual_event_participation::user_id.eq(user.id))
        .filter(individual_event_participation::event_id.eq(data.id))
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

pub async fn join_event_team(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<EventTeamAttendance>,
) -> Result<(), StatusCode> {
    let is_leader: bool = team_members::table
        .select(team_members::is_leader)
        .filter(team_members::team_id.eq(data.team_id))
        .filter(team_members::student_id.eq(user.id))
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::UNAUTHORIZED
        })?;
    if !is_leader {
        return Err(StatusCode::UNAUTHORIZED);
    }
    data.insert_into(team_event_participations::table)
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

pub async fn leave_event_team(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<EventTeamAttendance>,
) -> Result<(), StatusCode> {
    let is_leader: bool = team_members::table
        .select(team_members::is_leader)
        .filter(team_members::team_id.eq(data.team_id))
        .filter(team_members::student_id.eq(user.id))
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::UNAUTHORIZED
        })?;
    if !is_leader {
        return Err(StatusCode::UNAUTHORIZED);
    }
    diesel::delete(team_event_participations::table)
        .filter(team_event_participations::team_id.eq(data.team_id))
        .filter(team_event_participations::event_id.eq(data.event_id))
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

pub async fn joined_events_individual(
    State(state): State<SiteState>,
    user: User,
) -> Result<Json<Vec<Event>>, StatusCode> {
    individual_event_participation::table
        .inner_join(events::table)
        .select(Event::as_select())
        .filter(individual_event_participation::user_id.eq(user.id))
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

pub async fn joined_events_team(
    State(state): State<SiteState>,
    user: User,
    Query(data): Query<TeamId>,
) -> Result<Json<Vec<Event>>, StatusCode> {
    team_event_participations::table
        .inner_join(teams::table.inner_join(team_members::table))
        .filter(teams::id.eq(data.id))
        .inner_join(events::table)
        .select(Event::as_select())
        .filter(team_members::student_id.eq(user.id))
        .distinct()
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

pub async fn event_domain(
    State(state): State<SiteState>,
    Query(data): Query<EventId>,
) -> Result<Json<Domain>, StatusCode> {
    events::table
        .inner_join(domains::table)
        .select(Domain::as_select())
        .filter(events::id.eq(data.id))
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::NOT_MODIFIED
        })
        .map(|v| Json(v))
}
