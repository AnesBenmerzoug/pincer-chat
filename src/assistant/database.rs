pub mod models;
pub mod schema;

use std::fmt;

use anyhow::{anyhow, Error, Result};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_async::{AsyncConnection, RunQueryDsl};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use home::home_dir;

use super::notification::{Notifier, NotifierMessage};
use super::ollama::types::{Message as OllamaMessage, Role};

use self::models::{Message, NewMessage, NewThread, Thread};
use self::schema::{messages, threads};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct Database {
    database_url: String,
    connection: SyncConnectionWrapper<SqliteConnection>,
    pub notifier: Notifier,
}

impl fmt::Debug for Database {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Database")
    }
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
        let connection = Self::connect(database_url.clone()).await?;
        let instance = Self {
            database_url: String::from(database_url),
            connection,
            notifier: Notifier::new(),
        };
        Ok(instance)
    }

    pub async fn connect(database_url: &str) -> Result<SyncConnectionWrapper<SqliteConnection>> {
        let connection = SyncConnectionWrapper::<SqliteConnection>::establish(database_url).await?;
        Ok(connection)
    }

    pub async fn run_migrations(&self) -> Result<()> {
        let connection = Self::connect(&*self.database_url.clone()).await?;
        let mut async_wrapper: AsyncConnectionWrapper<SyncConnectionWrapper<SqliteConnection>> =
            AsyncConnectionWrapper::from(connection);
        tokio::task::spawn_blocking(move || {
            async_wrapper.run_pending_migrations(MIGRATIONS).unwrap();
        })
        .await?;
        Ok(())
    }

    pub async fn is_running(&mut self) -> bool {
        !schema::threads::table
            .select(Thread::as_select())
            .limit(1)
            .load(&mut self.connection)
            .await
            .is_err()
    }

    pub async fn create_thread(&mut self, title: &str) -> Result<Thread> {
        let new_thread = NewThread { title: title };
        let inserted_thread = diesel::insert_into(threads::table)
            .values(&new_thread)
            .returning(Thread::as_returning())
            .get_result(&mut self.connection)
            .await?;

        // System Message
        let system_message = NewMessage {
            thread_id: inserted_thread.id,
            content: "You are a helpful assistant. You reply to user queries in a helpful manner.\n \
            You should give concise responses to very simple questions, but provide thorough responses to more complex and open-ended questions. \
            You help with writing, analysis, question answering, math, coding, and all sorts of other tasks. \
            You use markdown formatting for your replies.",
            role: Role::System.into(),
        };

        diesel::insert_into(messages::table)
            .values(&system_message)
            .returning(Message::as_returning())
            .get_result(&mut self.connection)
            .await?;

        self.notifier
            .notify(NotifierMessage::NewThread(inserted_thread.clone()));
        Ok(inserted_thread)
    }

    pub async fn delete_thread(&mut self, thread_id: i64) -> Result<()> {
        use self::schema::threads::dsl::*;

        diesel::delete(threads.filter(id.eq(thread_id)))
            .execute(&mut self.connection)
            .await?;
        Ok(())
    }

    pub async fn get_threads(&mut self) -> Result<Vec<Thread>> {
        let threads = schema::threads::table
            .select(Thread::as_select())
            .order_by(threads::last_updated_at.desc())
            .load(&mut self.connection)
            .await?;
        Ok(threads)
    }

    pub async fn get_latest_thread(&mut self) -> Result<Option<Thread>> {
        let thread = schema::threads::table
            .select(Thread::as_select())
            .first(&mut self.connection)
            .await
            .optional();
        thread.map_err(Error::msg)
    }

    pub async fn get_messages(&mut self, thread_id: i64) -> Result<Vec<Message>> {
        let messages = schema::messages::table
            .filter(schema::messages::thread_id.eq(thread_id))
            .select(Message::as_select())
            .load(&mut self.connection)
            .await?;

        self.notifier
            .notify(NotifierMessage::GetThreadMessages(messages.clone()));
        Ok(messages)
    }

    pub async fn create_message(
        &mut self,
        thread_id: i64,
        content: String,
        role: Role,
    ) -> Result<Message> {
        let new_message = NewMessage {
            thread_id,
            content: &*content,
            role: role.into(),
        };
        let inserted_message = diesel::insert_into(messages::table)
            .values(&new_message)
            .returning(Message::as_returning())
            .get_result(&mut self.connection)
            .await?;

        self.notifier
            .notify(NotifierMessage::NewMessage(inserted_message.clone()));
        Ok(inserted_message)
    }

    pub async fn update_message(&mut self, message_id: i64, content_update: String) -> Result<()> {
        use self::schema::messages::dsl::*;

        diesel::update(messages.find(message_id))
            .set(content.eq(content.concat(&*content_update)))
            .execute(&mut self.connection)
            .await?;
        self.notifier
            .notify(NotifierMessage::UpdateMessage(content_update));
        Ok(())
    }
}
