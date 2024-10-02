use axum::{
    extract::{Query, State},
    Form, Json,
};
use diesel::prelude::*;
use http::StatusCode;

use crate::{
    forms::teams::{ChangeTeam, MemberId, NewTeamReq, TeamId},
    models::{
        team::{NewTeamRequest, Team, TeamMember, TeamRequest},
        users::User,
    },
    schema::{team_members, team_requests, teams, users},
    state::SiteState,
};

pub async fn get_teams(
    State(state): State<SiteState>,
    user: Option<User>,
    data: Option<Query<TeamId>>,
) -> Result<Json<Vec<Team>>, StatusCode> {
    if let Some(data) = data {
        teams::table
            .select(Team::as_select())
            .filter(teams::id.eq(data.id))
            .load(&mut state.connection.get().map_err(|e| {
                log::error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?)
            .map(|v| Json(v))
            .map_err(|e| {
                log::error!("{e:?}");
                StatusCode::NOT_FOUND
            })
    } else if let Some(user) = user {
        teams::table
            .inner_join(team_members::table)
            .select(Team::as_select())
            .filter(team_members::student_id.eq(user.id))
            .load(&mut state.connection.get().map_err(|e| {
                log::error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?)
            .map(|v| Json(v))
            .map_err(|e| {
                log::error!("{e:?}");
                StatusCode::NOT_FOUND
            })
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn create_team(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<NewTeamReq>,
) -> Result<(), StatusCode> {
    if !user.verified {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let team_id: i32 = data
        .insert_into(teams::table)
        .returning(teams::id)
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::CONFLICT
        })?;
    TeamMember {
        team_id,
        student_id: user.id,
        is_leader: true,
    }
    .insert_into(team_members::table)
    .returning(team_members::team_id)
    .get_result(&mut state.connection.get().map_err(|e| {
        log::error!("{e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?)
    .map(|_| ())
    .map_err(|e| {
        log::error!("{e:?}");
        StatusCode::CONFLICT
    });

    for member in data.members.into_iter() {
        let student_id = users::table
            .select(users::id)
            .filter(users::email.eq(member))
            .get_result(&mut state.connection.get().map_err(|e| {
                log::error!("{e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?)
            .map_err(|e| {
                log::error!("{e:?}");
                StatusCode::UNAUTHORIZED
            })?;
        TeamRequest {
            team_id,
            student_id,
        }
        .insert_into(team_requests::table)
        .execute(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map(|_| ())
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::UNAUTHORIZED
        })?;
    }
    Ok(())
}

pub async fn delete_team(
    State(state): State<SiteState>,
    user: User,
    Query(data): Query<TeamId>,
) -> Result<(), StatusCode> {
    if !user.verified {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let is_leader: bool = team_members::table
        .select(team_members::is_leader)
        .filter(team_members::team_id.eq(data.id))
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
    diesel::delete(teams::table)
        .filter(teams::id.eq(data.id))
        .execute(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map(|_| ())
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::UNAUTHORIZED
        })
}

pub async fn change_team(
    State(state): State<SiteState>,
    user: User,
    Query(data): Query<ChangeTeam>,
) -> Result<(), StatusCode> {
    if !user.verified {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let is_leader: bool = team_members::table
        .select(team_members::is_leader)
        .filter(team_members::team_id.eq(data.id))
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
    diesel::update(teams::table)
        .filter(teams::id.eq(data.id))
        .set(data)
        .execute(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map(|_| ())
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::UNAUTHORIZED
        })
}

pub async fn get_team_members(
    State(state): State<SiteState>,
    Query(data): Query<TeamId>,
) -> Result<Json<Vec<TeamMember>>, StatusCode> {
    team_members::table
        .select(TeamMember::as_select())
        .filter(team_members::team_id.eq(data.id))
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

pub async fn remove_member(
    State(state): State<SiteState>,
    user: User,
    Query(data): Query<MemberId>,
) -> Result<(), StatusCode> {
    if !user.verified {
        return Err(StatusCode::UNAUTHORIZED);
    }
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
    diesel::delete(team_members::table)
        .filter(team_members::team_id.eq(data.team_id))
        .filter(team_members::student_id.eq(data.student_id))
        .execute(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map(|_| ())
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::UNAUTHORIZED
        })
}

pub async fn get_team_request(
    State(state): State<SiteState>,
    user: User,
    Query(data): Query<TeamId>,
) -> Result<Json<Vec<TeamRequest>>, StatusCode> {
    let is_leader: bool = team_members::table
        .select(team_members::is_leader)
        .filter(team_members::team_id.eq(data.id))
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
    team_requests::table
        .select(TeamRequest::as_select())
        .filter(team_requests::team_id.eq(data.id))
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

pub async fn accept_team_request(
    State(state): State<SiteState>,
    user: User,
    Query(data): Query<TeamId>,
) -> Result<(), StatusCode> {
    if !user.verified {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let request: TeamRequest = team_requests::table
        .select(TeamRequest::as_select())
        .filter(team_requests::team_id.eq(data.id))
        .filter(team_requests::student_id.eq(user.id))
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::UNAUTHORIZED
        })?;
    TeamMember {
        team_id: request.team_id,
        student_id: request.student_id,
        is_leader: false,
    }
    .insert_into(team_members::table)
    .execute(&mut state.connection.get().map_err(|e| {
        log::error!("{e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?)
    .map(|_| ())
    .map_err(|e| {
        log::error!("{e:?}");
        StatusCode::UNAUTHORIZED
    })
}

pub async fn send_team_request(
    State(state): State<SiteState>,
    user: User,
    Query(data): Query<NewTeamRequest>,
) -> Result<(), StatusCode> {
    if !user.verified {
        return Err(StatusCode::UNAUTHORIZED);
    }
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
    let student_id = users::table
        .select(users::id)
        .filter(users::email.eq(data.email))
        .get_result(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map_err(|e| {
            log::error!("{e:?}");
            StatusCode::UNAUTHORIZED
        })?;
    TeamRequest {
        team_id: data.team_id,
        student_id,
    }
    .insert_into(team_requests::table)
    .execute(&mut state.connection.get().map_err(|e| {
        log::error!("{e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?)
    .map(|_| ())
    .map_err(|e| {
        log::error!("{e:?}");
        StatusCode::UNAUTHORIZED
    })
}
