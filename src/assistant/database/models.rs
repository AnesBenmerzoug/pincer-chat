use super::schema::{messages, threads};
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Insertable)]
#[diesel(table_name = threads)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewThread<'a> {
    pub title: &'a str,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = threads)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Thread {
    pub id: i64,
    pub title: String,
    pub created_at: NaiveDateTime,
    pub last_updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewMessage<'a> {
    pub thread_id: i64,
    pub content: &'a str,
    pub role: &'a str,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Message {
    pub id: i64,
    pub thread_id: i64,
    pub created_at: NaiveDateTime,
    pub content: String,
    pub role: String,
}
