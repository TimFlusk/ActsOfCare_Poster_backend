use anyhow::Result;
use rusqlite::{Connection, OptionalExtension, params};
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

    /// Fetch a single user by file_name.
    pub fn get_user(&self, file_name: &str) -> Result<Option<UserDetails>> {
        let conn = self.conn.lock().unwrap();
        let user = conn.query_row(
            "SELECT file_name, email, consent, save_location FROM users WHERE file_name = ?1",
            params![file_name],
            |row| {
                Ok(UserDetails {
                    file_name: row.get(0)?,
                    email: row.get(1)?,
                    consent: row.get::<_, i32>(2)? != 0,
                    save_location: row.get(3)?,
                })
            },
        ).optional()?;
        Ok(user)
    }

    /// Fetch all users.
    pub fn get_all_users(&self) -> Result<Vec<UserDetails>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT file_name, email, consent, save_location FROM users"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(UserDetails {
                file_name: row.get(0)?,
                email: row.get(1)?,
                consent: row.get::<_, i32>(2)? != 0,
                save_location: row.get(3)?,
            })
        })?;

        let mut users = Vec::new();
        for user in rows {
            users.push(user?);
        }
        Ok(users)
    }
}
