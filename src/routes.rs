use crate::auth::faculty_sign_up;
use crate::auth::logout;
use crate::auth::resend_email;
use crate::auth::sign_in;
use crate::auth::student_sign_up;
use crate::domain::add_domain_faculty_coordinator;
use crate::domain::add_domain_student_coordinator;
use crate::domain::change_domain;
use crate::domain::create_domain;
use crate::domain::delete_domain;
use crate::domain::get_domain;
use crate::domain::get_domain_faculty_coordinator;
use crate::domain::get_domain_photo;
use crate::domain::get_domain_student_coordinator;
use crate::domain::set_domain_photo;
use crate::event::add_event_coordinator;
use crate::event::change_event;
use crate::event::create_event;
use crate::event::delete_event;
use crate::event::event_domain;
use crate::event::get_event;
use crate::event::get_event_coordinator;
use crate::event::get_event_individual_attendance;
use crate::event::get_event_photo;
use crate::event::get_event_team_attendance;
use crate::event::get_events_by_domain;
use crate::event::join_event_individual;
use crate::event::join_event_team;
use crate::event::joined_events_individual;
use crate::event::joined_events_team;
use crate::event::leave_event_individual;
use crate::event::leave_event_team;
use crate::event::mark_event_individual_attendance;
use crate::event::mark_event_team_attendance;
use crate::event::remove_event_individual_attendance;
use crate::event::remove_event_team_attendance;
use crate::event::set_event_photo;
use crate::profile::change_profile;
use crate::profile::create_faculty_profile;
use crate::profile::create_student_profile;
use crate::profile::get_departments;
use crate::profile::get_faculty_profile;
use crate::profile::get_individual_team_requests;
use crate::profile::get_profile;
use crate::profile::get_profile_photo;
use crate::profile::get_student_profile;
use crate::profile::reset_password;
use crate::profile::send_reset_mail;
use crate::profile::set_profile_photo;
use crate::profile::verify_user;
use crate::state::SiteState;
use crate::team::accept_team_request;
use crate::team::change_team;
use crate::team::create_team;
use crate::team::delete_team;
use crate::team::get_team_members;
use crate::team::get_team_request;
use crate::team::get_teams;
use crate::team::reject_team_request;
use crate::team::remove_member;
use crate::team::send_team_request;
use crate::workshop::add_workshop_coordinator;
use crate::workshop::change_workshop;
use crate::workshop::create_workshop;
use crate::workshop::delete_workshop;
use crate::workshop::get_workshop;
use crate::workshop::get_workshop_attendance;
use crate::workshop::get_workshop_coordinator;
use crate::workshop::get_workshop_photo;
use crate::workshop::join_workshop;
use crate::workshop::joined_workshops_individual;
use crate::workshop::leave_workshop_individual;
use crate::workshop::mark_workshop_attendance;
use crate::workshop::set_workshop_photo;
use axum::{
    routing::{get, post},
    Router,
};

pub fn setup_routes() -> Router<SiteState> {
    Router::new()
        .route("/auth/sign_in", post(sign_in))
        .route("/auth/logout", get(logout))
        .route("/auth/student/sign_up", post(student_sign_up))
        .route("/auth/faculty/sign_up", post(faculty_sign_up))
        .route("/auth/verify", get(verify_user).post(resend_email))
        .route("/profile/password_reset", post(reset_password).put(send_reset_mail))
        .route("/profile", get(get_profile).patch(change_profile))
        .route(
            "/profile/student",
            get(get_student_profile).post(create_student_profile),
        )
        .route(
            "/profile/faculty",
            get(get_faculty_profile).post(create_faculty_profile),
        )
        .route(
            "/profile/photo",
            get(get_profile_photo).post(set_profile_photo),
        )
        .route("/profile/requests", get(get_individual_team_requests))
        .route(
            "/domain",
            get(get_domain)
                .post(create_domain)
                .delete(delete_domain)
                .patch(change_domain),
        )
        .route("/domain/event", get(get_events_by_domain))
        .route(
            "/domain/coordinator/faculty",
            get(get_domain_faculty_coordinator).post(add_domain_faculty_coordinator),
        )
        .route(
            "/domain/coordinator/student",
            get(get_domain_student_coordinator).post(add_domain_student_coordinator),
        )
        .route(
            "/domain/photo",
            get(get_domain_photo).post(set_domain_photo),
        )
        .route(
            "/event",
            get(get_event)
                .post(create_event)
                .delete(delete_event)
                .patch(change_event),
        )
        .route(
            "/event/coordinator",
            get(get_event_coordinator).post(add_event_coordinator),
        )
        .route(
            "/event/attendance/individual",
            get(get_event_individual_attendance)
                .post(mark_event_individual_attendance)
                .delete(remove_event_individual_attendance),
        )
        .route(
            "/event/attendance/team",
            get(get_event_team_attendance)
                .post(mark_event_team_attendance)
                .delete(remove_event_team_attendance),
        )
            .route("/event/domain", get(event_domain))
        .route("/event/photo", get(get_event_photo).post(set_event_photo))
        .route(
            "/event/join/individual",
            post(join_event_individual).delete(leave_event_individual),
        )
        .route(
            "/event/join/team",
            post(join_event_team).delete(leave_event_team),
        )
        .route("/event/joined/individual", get(joined_events_individual))
        .route("/event/joined/team", get(joined_events_team))
        .route(
            "/workshop",
            get(get_workshop)
                .post(create_workshop)
                .delete(delete_workshop)
                .patch(change_workshop),
        )
        .route(
            "/workshop/coordinator",
            get(get_workshop_coordinator).post(add_workshop_coordinator),
        )
        .route(
            "/workshop/photo",
            get(get_workshop_photo).post(set_workshop_photo),
        )
        .route("/workshop/join", post(join_workshop).delete(leave_workshop_individual))
        .route(
            "/workshop/attendance",
            get(get_workshop_attendance).post(mark_workshop_attendance),
        )
        .route(
            "/workshop/joined/individual",
            get(joined_workshops_individual),
        )
        .route(
            "/team",
            get(get_teams)
                .post(create_team)
                .delete(delete_team)
                .patch(change_team),
        )
        .route("/team/member", get(get_team_members).delete(remove_member))
        .route(
            "/team/request",
            get(get_team_request)
                .post(send_team_request)
                .delete(reject_team_request)
                .put(accept_team_request),
        )
        .route("/departments", get(get_departments))
}
