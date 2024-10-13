use axum::{
    extract::{Query, State},
    Json,
};
use axum_extra::extract::Form;
use diesel::prelude::*;
use diesel::result::Error;
use http::StatusCode;

use crate::{
    forms::teams::{ChangeTeam, MemberId, NewTeamReq, TeamId, TeamName},
    models::{
        team::{NewTeamRequest, Team, TeamMember, TeamMemberResp, TeamRequest, TeamResponse},
        users::User,
    },
    schema::{students, team_members, team_requests, teams, users},
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
        log::info!("{user:#?}");
        team_members::table
            .inner_join(teams::table)
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
    axum_extra::extract::Form(data): axum_extra::extract::Form<NewTeamReq>,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    state
        .connection
        .get()
        .map_err(|e| {
            log::error!("at line {} {e:?}", line!());
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .transaction::<_, Error, _>(|connection| {
            let team_id: i32 = TeamName { name: data.name }
                .insert_into(teams::table)
                .returning(teams::id)
                .get_result(connection)
                .map_err(|e| {
                    log::error!("{e:?}");
                    e
                })?;
            TeamMember {
                team_id,
                student_id: user.id,
                is_leader: true,
            }
            .insert_into(team_members::table)
            .execute(connection)
            .map(|_| ())
            .map_err(|e| {
                log::error!("{e:?}");
                e
            })?;

            log::info!("{:?}", data.members.clone());
            if let Some(v) = data.members.first() {
                if !v.is_empty() && data.members.len() < 4 {
                    for member in data.members.into_iter() {
                        let student_id = users::table
                            .select(users::id)
                            .filter(users::email.eq(member.trim_ascii()))
                            .get_result(connection)
                            .map_err(|e| {
                                log::error!("at line {} {e:?}", line!());
                                e
                            })?;
                        TeamRequest {
                            team_id,
                            student_id,
                        }
                        .insert_into(team_requests::table)
                        .execute(connection)
                        .map(|_| ())
                        .map_err(|e| {
                            log::error!("at line {} {e:?}", line!());
                            e
                        })?;
                    }
                }
            }
            Ok(())
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

pub async fn delete_team(
    State(state): State<SiteState>,
    user: User,
    Query(data): Query<TeamId>,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
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
    if !user.verified || !user.is_payment_done(&state.connection) {
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
) -> Result<Json<Vec<TeamMemberResp>>, StatusCode> {
    team_members::table
        .inner_join(teams::table)
        .inner_join(students::table.inner_join(users::table))
        .select((
            Team::as_select(),
            User::as_select(),
            TeamMember::as_select(),
        ))
        .filter(team_members::team_id.eq(data.id))
        .load(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map(|v| {
            Json(
                v.into_iter()
                    .map(|(team, user, member)| TeamMemberResp {
                        team_id: team.id,
                        student_id: user.id,
                        is_leader: member.is_leader,
                        team_name: team.name,
                        name: user.name,
                        verified: user.verified,
                        email: user.email,
                    })
                    .collect::<Vec<TeamMemberResp>>(),
            )
        })
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
    if !user.verified || !user.is_payment_done(&state.connection) {
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
    data: Option<Query<TeamId>>,
) -> Result<Json<Vec<TeamResponse>>, StatusCode> {
    let query =
        team_requests::table
            .inner_join(teams::table.inner_join(
                team_members::table.inner_join(students::table.inner_join(users::table)),
            ))
            .select((User::as_select(), Team::as_select()))
            .into_boxed();
    let query = if let Some(Query(data)) = data {
        query.filter(team_requests::team_id.eq(data.id))
    } else {
        query
    };
    query
        .filter(team_requests::student_id.eq(user.id))
        .load(&mut state.connection.get().map_err(|e| {
            log::error!("{e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
        .map(|v| {
            Json(
                v.into_iter()
                    .map(|(user, team)| TeamResponse {
                        team_name: team.name,
                        team_id: team.id,
                        leader_name: user.name,
                    })
                    .collect::<Vec<TeamResponse>>(),
            )
        })
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
    if !user.verified || !user.is_payment_done(&state.connection) {
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
    })?;
    diesel::delete(team_requests::table)
        .filter(team_requests::team_id.eq(data.id))
        .filter(team_requests::student_id.eq(user.id))
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

pub async fn send_team_request(
    State(state): State<SiteState>,
    user: User,
    Form(data): Form<NewTeamRequest>,
) -> Result<(), StatusCode> {
    if !user.verified || !user.is_payment_done(&state.connection) {
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
        StatusCode::CONFLICT
    })
}

pub async fn reject_team_request(
    State(state): State<SiteState>,
    user: User,
    Query(data): Query<TeamId>,
) -> Result<Json<TeamRequest>, StatusCode> {
    log::info!("{:#?}", user);
    log::info!("{:#?}", data);
    diesel::delete(team_requests::table)
        .filter(team_requests::team_id.eq(data.id))
        .filter(team_requests::student_id.eq(user.id))
        .returning(TeamRequest::as_select())
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
