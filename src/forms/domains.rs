use serde::Deserialize;

use diesel::prelude::*;
#[derive(Deserialize, Insertable, Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::domains)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateDomain {
    pub name: String,
    pub description: String,
}

#[derive(Queryable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::domains)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeleteDomain {
    pub id: i32,
}

#[derive(Deserialize, AsChangeset, Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::domains)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChangeDomain {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Queryable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::domains)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GetDomainPhoto {
    pub id: i32,
}


#[derive(Queryable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::domains)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GetDomainFacultyCoordinator {
    pub id: i32,
}

#[derive(Queryable, Insertable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::faculty_coordinators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AddDomainFacultyCoordinator {
    pub faculty_id: i32,
    pub domain_id: i32,
}

#[derive(Queryable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::domains)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GetDomainStudentCoordinator {
    pub id: i32,
}

#[derive(Queryable, Insertable, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::student_domain_coordinators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AddDomainStudentCoordinator {
    pub student_id: i32,
    pub domain_id: i32,
}

