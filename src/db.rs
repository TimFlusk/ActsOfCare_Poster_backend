use anyhow::Result;
use rusqlite::{Connection, params};
use std::sync::Mutex;

use crate::models::UserDetails;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        conn.execute_batch("
            PRAGMA journal_mode=WAL;
            CREATE TABLE IF NOT EXISTS users (
                file_name    TEXT PRIMARY KEY,
                email        TEXT NOT NULL,
                consent      INTEGER NOT NULL,
                save_location TEXT NOT NULL
            );
        ")?;

        tracing::info!("SQLite database opened at {}", path);
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Insert or replace a user record, keyed on file_name.
    pub fn upsert_user(&self, user: &UserDetails) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO users (file_name, email, consent, save_location)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(file_name) DO UPDATE SET
               email         = excluded.email,
               consent       = excluded.consent,
               save_location = excluded.save_location",
            params![
                user.file_name,
                user.email,
                user.consent as i32,
                user.save_location,
            ],
        )?;
        tracing::debug!("Upserted user: {}", user.file_name);
        Ok(())
    }
}
