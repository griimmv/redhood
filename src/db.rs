use anyhow::Result;
use rusqlite::{params, Connection};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS state (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sent_notifications (
                id        TEXT PRIMARY KEY,
                source    TEXT NOT NULL,
                sent_at   TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;
        Ok(())
    }

    pub fn get_state(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM state WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get::<_, String>(0)?)),
            None => Ok(None),
        }
    }

    pub fn set_state(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO state (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn is_notification_sent(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT 1 FROM sent_notifications WHERE id = ?1")?;
        Ok(stmt.exists(params![id])?)
    }

    pub fn mark_notification_sent(&self, id: &str, source: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO sent_notifications (id, source) VALUES (?1, ?2)",
            params![id, source],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Database {
        let db = Database::open(":memory:").unwrap();
        db.migrate().unwrap();
        db
    }

    #[test]
    fn open_and_migrate() {
        let db = Database::open(":memory:");
        assert!(db.is_ok());
        assert!(db.unwrap().migrate().is_ok());
    }

    #[test]
    fn get_state_missing_key() {
        let db = db();
        let val = db.get_state("nonexistent").unwrap();
        assert_eq!(val, None);
    }

    #[test]
    fn set_state_roundtrip() {
        let db = db();
        db.set_state("k1", "v1").unwrap();
        assert_eq!(db.get_state("k1").unwrap(), Some("v1".into()));
    }

    #[test]
    fn set_state_overwrite() {
        let db = db();
        db.set_state("k", "old").unwrap();
        db.set_state("k", "new").unwrap();
        assert_eq!(db.get_state("k").unwrap(), Some("new".into()));
    }

    #[test]
    fn is_notification_sent_absent() {
        let db = db();
        assert!(!db.is_notification_sent("nope").unwrap());
    }

    #[test]
    fn mark_and_check_notification_sent() {
        let db = db();
        assert!(!db.is_notification_sent("msg_1").unwrap());
        db.mark_notification_sent("msg_1", "reddit").unwrap();
        assert!(db.is_notification_sent("msg_1").unwrap());
    }
}
