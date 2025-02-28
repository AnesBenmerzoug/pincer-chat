pub mod models;
pub mod schema;

use anyhow::{anyhow, Result};
use diesel::prelude::*;
use diesel::sqlite::{Sqlite, SqliteConnection};
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_async::{AsyncConnection, RunQueryDsl};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use home::home_dir;

use self::models::{NewMessage, Message, NewThread, Thread};

use super::ollama::types::Role;

type InnerConnection = SqliteConnection;
type InnerDB = Sqlite;
const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct Database {
    connection: SyncConnectionWrapper<SqliteConnection>,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let mut database_url = match home_dir() {
            Some(path) if !path.as_os_str().is_empty() => path,
            _ => return Err(anyhow!("Unable to get home dir!")),
        };
        database_url.push(".pincer_chat");
        database_url.push("database.db");
        let database_url = match database_url.as_path().to_str() {
            Some(path) => path,
            None => return Err(anyhow!("Unable to get your home dir!")),
        };
        let connection = SyncConnectionWrapper::<SqliteConnection>::establish(database_url).await?;
        let instance = Self { connection };
        Ok(instance)
    }

    pub async fn create_thread(&mut self, title: String) -> Result<Thread> {
        let new_thread = NewThread { title: &*title };
        let inserted_thread = diesel::insert_into(schema::threads::table)
            .values(&new_thread)
            .returning(Thread::as_returning())
            .get_result(&mut self.connection)
            .await?;
        Ok(inserted_thread)
    }

    pub async fn delete_thread(&mut self, thread_id: i64) -> Result<()> {
        diesel::delete(schema::threads::table.filter(schema::threads::id.eq(thread_id)))
            .execute(&mut self.connection)
            .await?;
        Ok(())
    }

    pub async fn get_messages(&mut self, thread_id: i64) -> Result<Vec<Message>> {
        let messages = schema::messages::table
            .filter(schema::messages::thread_id.eq(thread_id))
            .select(Message::as_select())
            .load(&mut self.connection)
            .await?;

        Ok(messages)
    }

    pub async fn create_message(&mut self, thread_id: i64, content: String, role: Role) -> Result<Message> {
        let role = match role {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::System => "system",
            Role::Tool => "tool"
        };
        let new_message = NewMessage { thread_id, content: &*content, role};
        let inserted_message = diesel::insert_into(schema::messages::table)
            .values(&new_message)
            .returning(Message::as_returning())
            .get_result(&mut self.connection)
            .await?;
        Ok(inserted_message)
    }
}
