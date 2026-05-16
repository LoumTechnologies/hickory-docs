use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &str) -> Result<Self> {
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("database lock poisoned")
    }

    pub fn run_migrations(&self) -> Result<()> {
        let conn = self.conn();
        let migration_dir = Path::new("migrations");
        if !migration_dir.exists() {
            return Ok(());
        }
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                name TEXT PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        )?;
        let mut entries: Vec<_> = std::fs::read_dir(migration_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let already_applied: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM _migrations WHERE name = ?1",
                [&name],
                |row| row.get(0),
            )?;
            if !already_applied {
                let sql = std::fs::read_to_string(entry.path())?;
                conn.execute_batch(&sql)?;
                conn.execute("INSERT INTO _migrations (name) VALUES (?1)", [&name])?;
                tracing::info!("applied migration: {}", name);
            }
        }
        Ok(())
    }
}
