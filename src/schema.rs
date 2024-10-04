// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "department"))]
    pub struct Department;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "mode"))]
    pub struct Mode;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "participation_type"))]
    pub struct ParticipationType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "role"))]
    pub struct Role;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "title"))]
    pub struct Title;
}

diesel::table! {
    domains (id) {
        id -> Int4,
        name -> Text,
        description -> Text,
        photo_hash -> Nullable<Bytea>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Mode;
    use super::sql_types::ParticipationType;

    events (id) {
        id -> Int4,
        name -> Text,
        description -> Text,
        mode -> Mode,
        venue -> Text,
        domain_id -> Int4,
        prize -> Int4,
        points -> Int4,
        ps_link -> Text,
        start_time -> Timestamp,
        end_time -> Timestamp,
        registeration_start -> Timestamp,
        registeration_end -> Timestamp,
        whatsapp_link -> Text,
        photo_hash -> Nullable<Bytea>,
        participation_type -> ParticipationType,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Department;
    use super::sql_types::Title;

    faculty (user_id) {
        user_id -> Int4,
        dept -> Department,
        title -> Title,
    }
}

diesel::table! {
    faculty_coordinators (faculty_id, domain_id) {
        faculty_id -> Int4,
        domain_id -> Int4,
    }
}

diesel::table! {
    individual_event_participation (event_id, user_id) {
        event_id -> Int4,
        user_id -> Int4,
        attended -> Bool,
    }
}

diesel::table! {
    payments (payment_id) {
        user_id -> Int4,
        payment_id -> Text,
        payment_amount -> Int4,
        verified -> Bool,
    }
}

diesel::table! {
    sponsors (id) {
        id -> Int4,
        name -> Text,
        photo_hash -> Bytea,
    }
}

diesel::table! {
    student_domain_coordinators (student_id, domain_id) {
        student_id -> Int4,
        domain_id -> Int4,
    }
}

diesel::table! {
    student_event_coordinators (student_id, event_id) {
        student_id -> Int4,
        event_id -> Int4,
    }
}

diesel::table! {
    student_workshop_coordinators (student_id, workshop_id) {
        student_id -> Int4,
        workshop_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Department;

    students (user_id) {
        user_id -> Int4,
        college -> Text,
        reg_no -> Text,
        dept -> Department,
    }
}

diesel::table! {
    team_event_participations (team_id, event_id) {
        team_id -> Int4,
        event_id -> Int4,
        attended -> Bool,
    }
}

diesel::table! {
    team_members (team_id, student_id) {
        team_id -> Int4,
        student_id -> Int4,
        is_leader -> Bool,
    }
}

diesel::table! {
    team_requests (team_id, student_id) {
        team_id -> Int4,
        student_id -> Int4,
    }
}

diesel::table! {
    teams (id) {
        id -> Int4,
        name -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Role;

    users (id) {
        id -> Int4,
        name -> Text,
        dob -> Date,
        email -> Text,
        phone -> Text,
        role -> Role,
        photo_hash -> Nullable<Bytea>,
        verified -> Bool,
        password_hash -> Text,
    }
}

diesel::table! {
    workshop_participation (workshop_id, user_id) {
        workshop_id -> Int4,
        user_id -> Int4,
        attended -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Mode;

    workshops (id) {
        id -> Int4,
        name -> Text,
        description -> Text,
        mode -> Mode,
        venue -> Text,
        domain_id -> Int4,
        points -> Int4,
        ps_link -> Text,
        prof_name -> Text,
        prof_title -> Text,
        start_time -> Timestamp,
        end_time -> Timestamp,
        registeration_start -> Timestamp,
        registeration_end -> Timestamp,
        whatsapp_link -> Text,
        photo_hash -> Nullable<Bytea>,
    }
}

diesel::joinable!(events -> domains (domain_id));
diesel::joinable!(faculty -> users (user_id));
diesel::joinable!(faculty_coordinators -> domains (domain_id));
diesel::joinable!(faculty_coordinators -> faculty (faculty_id));
diesel::joinable!(individual_event_participation -> events (event_id));
diesel::joinable!(individual_event_participation -> users (user_id));
diesel::joinable!(payments -> users (user_id));
diesel::joinable!(student_domain_coordinators -> domains (domain_id));
diesel::joinable!(student_domain_coordinators -> students (student_id));
diesel::joinable!(student_event_coordinators -> events (event_id));
diesel::joinable!(student_event_coordinators -> students (student_id));
diesel::joinable!(student_workshop_coordinators -> students (student_id));
diesel::joinable!(student_workshop_coordinators -> workshops (workshop_id));
diesel::joinable!(students -> users (user_id));
diesel::joinable!(team_event_participations -> events (event_id));
diesel::joinable!(team_event_participations -> teams (team_id));
diesel::joinable!(team_members -> students (student_id));
diesel::joinable!(team_members -> teams (team_id));
diesel::joinable!(team_requests -> students (student_id));
diesel::joinable!(team_requests -> teams (team_id));
diesel::joinable!(workshop_participation -> users (user_id));
diesel::joinable!(workshop_participation -> workshops (workshop_id));
diesel::joinable!(workshops -> domains (domain_id));

diesel::allow_tables_to_appear_in_same_query!(
    domains,
    events,
    faculty,
    faculty_coordinators,
    individual_event_participation,
    payments,
    sponsors,
    student_domain_coordinators,
    student_event_coordinators,
    student_workshop_coordinators,
    students,
    team_event_participations,
    team_members,
    team_requests,
    teams,
    users,
    workshop_participation,
    workshops,
);
