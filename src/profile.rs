use base64::prelude::*;
use once_cell::sync::Lazy;
use std::fs::File;
use std::io::Cursor;

use axum::body::{Body, Bytes};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::{Form, Json};
use axum_macros::debug_handler;
use diesel::prelude::*;
use highway::HighwayHash;
use http::{header, HeaderMap, StatusCode};
use tokio_util::io::ReaderStream;

use crate::forms::teams::TeamId;
use crate::forms::users::{
    ChangeProfile, GetProfilePhoto, Profile, VerificationClaims, VerificationQuery,
};
use crate::models::team::TeamRequest;
use crate::models::users::User;
use crate::schema::{team_requests, users};
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
    photo.save_with_format(
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
    );
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
        })?);
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

pub async fn verify_user(
    State(state): State<SiteState>,
    Query(data): Query<VerificationQuery>,
) -> Result<(), StatusCode> {
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
            .map(|_| ())
            .map_err(|e| {
                log::error!("{e:?}");
                StatusCode::UNAUTHORIZED
            })
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
