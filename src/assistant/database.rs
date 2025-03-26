pub mod models;
pub mod schema;

use std::fmt;

use anyhow::{anyhow, Result};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_async::{AsyncConnection, RunQueryDsl};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use home::home_dir;

use super::notification::{DatabaseNotifier, DatabaseNotifierMessage};
use super::ollama::types::Role;
use super::prompts::ASSISTANT_SYSTEM_PROMPT;

use self::models::{Message, NewMessage, NewThread, Thread};
use self::schema::{messages, threads};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct Database {
    database_url: String,
    connection: SyncConnectionWrapper<SqliteConnection>,
    pub notifier: DatabaseNotifier,
}

impl fmt::Debug for Database {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Database")
    }
}

impl Database {
    pub async fn new(database_url: Option<String>) -> Result<Self> {
        let database_url = match database_url {
            Some(database_url) => database_url,
            None => {
                let mut database_path = match home_dir() {
                    Some(path) if !path.as_os_str().is_empty() => path,
                    _ => panic!("Unable to get home dir!"),
                };
                database_path.push(".pincer_chat");
                database_path.push("database.db");

                match database_path.into_os_string().into_string() {
                    Ok(database_url) => database_url,
                    Err(error) => panic!("Unable to get your home dir because of: {:?}", error),
                }
            }
        };
        let connection = Self::connect(&database_url).await?;
        let instance = Self {
            database_url,
            connection,
            notifier: DatabaseNotifier::new(),
        };
        Ok(instance)
    }

    pub async fn connect(database_url: &str) -> Result<SyncConnectionWrapper<SqliteConnection>> {
        let connection = SyncConnectionWrapper::<SqliteConnection>::establish(database_url).await?;
        Ok(connection)
    }

    pub async fn run_migrations(&self) -> Result<()> {
        let connection = Self::connect(&self.database_url).await?;
        let mut async_wrapper: AsyncConnectionWrapper<SyncConnectionWrapper<SqliteConnection>> =
            AsyncConnectionWrapper::from(connection);
        let _ = tokio::task::spawn_blocking(move || -> Result<()> {
            match async_wrapper.run_pending_migrations(MIGRATIONS) {
                Ok(_) => {
                    tracing::info!("Successfully applied migraitons");
                    Ok(())
                }
                Err(error) => {
                    tracing::error!("Applying migrations failed because of: {error}");
                    Err(anyhow!("Applying migrations failed because of: {error}"))
                }
            }
        })
        .await?;
        Ok(())
    }

    pub async fn create_thread(&mut self, title: &str) -> Result<Thread> {
        let new_thread = NewThread { title };
        let inserted_thread = diesel::insert_into(threads::table)
            .values(&new_thread)
            .returning(Thread::as_returning())
            .get_result(&mut self.connection)
            .await?;

        // System Message
        let system_message = NewMessage {
            thread_id: inserted_thread.id,
            content: ASSISTANT_SYSTEM_PROMPT,
            role: Role::System.into(),
        };

        diesel::insert_into(messages::table)
            .values(&system_message)
            .returning(Message::as_returning())
            .get_result(&mut self.connection)
            .await?;

        self.notifier
            .notify(DatabaseNotifierMessage::NewThread(inserted_thread.clone()));
        Ok(inserted_thread)
    }

    pub async fn update_thread_title(&mut self, id: i64, title: String) -> Result<()> {
        use self::schema::threads::dsl;

        let updated_thread = diesel::update(dsl::threads.find(id))
            .set(dsl::title.eq(&*title))
            .get_result(&mut self.connection)
            .await?;
        self.notifier
            .notify(DatabaseNotifierMessage::UpdateThread(updated_thread));
        Ok(())
    }

    pub async fn delete_thread(&mut self, id: i64) -> Result<()> {
        use self::schema::threads::dsl;

        diesel::delete(dsl::threads.filter(dsl::id.eq(id)))
            .execute(&mut self.connection)
            .await?;
        Ok(())
    }

    pub async fn get_thread(&mut self, id: i64) -> Result<Thread> {
        use self::schema::threads::dsl;

        let thread = dsl::threads.find(id).first(&mut self.connection).await?;
        Ok(thread)
    }

    pub async fn get_threads(&mut self) -> Result<Vec<Thread>> {
        let threads = schema::threads::table
            .select(Thread::as_select())
            .order_by(threads::last_updated_at.desc())
            .load(&mut self.connection)
            .await?;
        Ok(threads)
    }

    pub async fn get_messages(&mut self, thread_id: i64) -> Result<Vec<Message>> {
        let messages = schema::messages::table
            .filter(schema::messages::thread_id.eq(thread_id))
            .select(Message::as_select())
            .load(&mut self.connection)
            .await?;

        self.notifier
            .notify(DatabaseNotifierMessage::GetThreadMessages(messages.clone()));
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
            content: &content,
            role: role.into(),
        };
        let inserted_message = diesel::insert_into(messages::table)
            .values(&new_message)
            .returning(Message::as_returning())
            .get_result(&mut self.connection)
            .await?;

        self.notifier.notify(DatabaseNotifierMessage::NewMessage(
            inserted_message.clone(),
        ));
        Ok(inserted_message)
    }

    pub async fn update_message(&mut self, message_id: i64, content_update: String) -> Result<()> {
        use self::schema::messages::dsl::*;

        diesel::update(messages.find(message_id))
            .set(content.eq(content.concat(&*content_update)))
            .execute(&mut self.connection)
            .await?;
        self.notifier
            .notify(DatabaseNotifierMessage::UpdateMessage(content_update));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rand::distr::{Alphanumeric, SampleString};

    use super::*;

    struct TestDatabaseWrapper {
        database_filepath: String,
        pub database: Database,
    }

    impl TestDatabaseWrapper {
        async fn setup() -> Self {
            let mut temp_dir = std::env::temp_dir();
            let database_filename: String =
                Alphanumeric.sample_string(&mut rand::rng(), 16) + ".db";
            temp_dir.push(database_filename);
            let database_url = match temp_dir.into_os_string().into_string() {
                Ok(database_url) => database_url,
                Err(_) => panic!("Unable to get temporary dir for tests!"),
            };
            let database = Database::new(Some(database_url.clone()))
                .await
                .expect("Instantiating database should work");
            database
                .run_migrations()
                .await
                .expect("Migrations should work");
            Self {
                database_filepath: database_url,
                database,
            }
        }
    }

    impl Drop for TestDatabaseWrapper {
        fn drop(&mut self) {
            std::fs::remove_file(&*self.database_filepath)
                .expect("Deleting database file should work");
        }
    }

    #[tokio::test]
    async fn test_creating_thread() {
        let mut database_wrapper = TestDatabaseWrapper::setup().await;
        let database = &mut database_wrapper.database;
        let result = database.create_thread("Test Thread Title").await;
        assert!(result.is_ok(), "Error: {}", result.err().unwrap());
        let thread = result.unwrap();
        assert!(thread.id > 0);
        assert_eq!(thread.title, "Test Thread Title");

        let messages = database
            .get_messages(thread.id)
            .await
            .expect("Getting thread message should work");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "system");
    }

    #[tokio::test]
    async fn test_updating_thread_title() {
        let mut database_wrapper = TestDatabaseWrapper::setup().await;
        let database = &mut database_wrapper.database;
        let result = database.create_thread("Test Thread Title").await;
        assert!(result.is_ok());
        let thread = result.unwrap();
        assert!(thread.id > 0);
        assert_eq!(thread.title, "Test Thread Title");

        database
            .update_thread_title(thread.id, String::from("A Different Title"))
            .await
            .expect("Updating thread title should work");

        let thread = database
            .get_thread(thread.id)
            .await
            .expect("Getting thread by id should work");
        assert_eq!(thread.title, "A Different Title");
    }
}
