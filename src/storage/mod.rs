use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const CURRENT_SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone)]
pub struct PersistentStore {
    db_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SnapshotMeta {
    pub name: String,
    pub checksum: String,
    pub updated_at: String,
}

impl PersistentStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let db_path = path.as_ref().to_path_buf();
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
        Self::run_migrations(&conn)?;
        Ok(Self { db_path })
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn schema_version(&self) -> Result<i64, String> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())
    }

    pub fn save_snapshot<T: Serialize>(&self, name: &str, value: &T) -> Result<SnapshotMeta, String> {
        let payload = serde_json::to_string(value).map_err(|e| e.to_string())?;
        let checksum = checksum_hex(&payload);
        let updated_at = Utc::now().to_rfc3339();

        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO state_snapshots (name, payload, checksum, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(name) DO UPDATE SET
                payload = excluded.payload,
                checksum = excluded.checksum,
                updated_at = excluded.updated_at
            "#,
            params![name, payload, checksum, updated_at],
        )
        .map_err(|e| e.to_string())?;

        Ok(SnapshotMeta {
            name: name.to_string(),
            checksum,
            updated_at,
        })
    }

    pub fn load_snapshot<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>, String> {
        let conn = self.connect()?;
        let row: Option<(String, String)> = conn
            .query_row(
                "SELECT payload, checksum FROM state_snapshots WHERE name = ?1",
                params![name],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        let Some((payload, checksum)) = row else {
            return Ok(None);
        };

        let actual = checksum_hex(&payload);
        if actual != checksum {
            return Err("snapshot checksum mismatch".into());
        }

        let decoded = serde_json::from_str::<T>(&payload).map_err(|e| e.to_string())?;
        Ok(Some(decoded))
    }

    pub fn list_snapshots(&self) -> Result<Vec<SnapshotMeta>, String> {
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare("SELECT name, checksum, updated_at FROM state_snapshots ORDER BY name")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(SnapshotMeta {
                    name: row.get(0)?,
                    checksum: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn backup_to<P: AsRef<Path>>(&self, backup_path: P) -> Result<(), String> {
        let src = self.db_path();
        let dst = backup_path.as_ref();

        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        fs::copy(src, dst).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn connect(&self) -> Result<Connection, String> {
        Connection::open(&self.db_path).map_err(|e| e.to_string())
    }

    fn run_migrations(conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS state_snapshots (
                name TEXT PRIMARY KEY,
                payload TEXT NOT NULL,
                checksum TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )
        .map_err(|e| e.to_string())?;

        let version: Option<i64> = conn
            .query_row(
                "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        if version.is_none() {
            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![CURRENT_SCHEMA_VERSION, Utc::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}

fn checksum_hex(payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    struct DemoState {
        node_id: String,
        open_orders: u64,
        escrow_total_axi: u64,
    }

    fn temp_db_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("axi_{}_{}.db", label, Uuid::new_v4()))
    }

    #[test]
    fn initializes_schema_and_version() {
        let path = temp_db_path("init");
        let store = PersistentStore::open(&path).unwrap();
        let version = store.schema_version().unwrap();

        assert_eq!(version, CURRENT_SCHEMA_VERSION);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn saves_and_loads_snapshot() {
        let path = temp_db_path("save_load");
        let store = PersistentStore::open(&path).unwrap();

        let state = DemoState {
            node_id: "axi-node-1".into(),
            open_orders: 3,
            escrow_total_axi: 125,
        };

        store.save_snapshot("runtime_state", &state).unwrap();
        let restored: Option<DemoState> = store.load_snapshot("runtime_state").unwrap();

        assert_eq!(restored, Some(state));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn backup_creates_copy() {
        let path = temp_db_path("backup_src");
        let backup = temp_db_path("backup_dst");
        let store = PersistentStore::open(&path).unwrap();

        let state = DemoState {
            node_id: "axi-node-2".into(),
            open_orders: 1,
            escrow_total_axi: 42,
        };

        store.save_snapshot("runtime_state", &state).unwrap();
        store.backup_to(&backup).unwrap();

        let backup_store = PersistentStore::open(&backup).unwrap();
        let restored: Option<DemoState> = backup_store.load_snapshot("runtime_state").unwrap();
        assert_eq!(restored, Some(state));

        let _ = fs::remove_file(path);
        let _ = fs::remove_file(backup);
    }

    #[test]
    fn checksum_detects_tampered_snapshot() {
        let path = temp_db_path("tamper");
        let store = PersistentStore::open(&path).unwrap();

        let state = DemoState {
            node_id: "axi-node-3".into(),
            open_orders: 9,
            escrow_total_axi: 999,
        };

        store.save_snapshot("runtime_state", &state).unwrap();

        let conn = store.connect().unwrap();
        conn.execute(
            "UPDATE state_snapshots SET payload = ?1 WHERE name = ?2",
            params![r#"{"node_id":"evil","open_orders":0,"escrow_total_axi":0}"#, "runtime_state"],
        )
        .unwrap();

        let result: Result<Option<DemoState>, String> = store.load_snapshot("runtime_state");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("checksum mismatch"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn restart_recovers_last_known_state() {
        let path = temp_db_path("restart");
        {
            let store = PersistentStore::open(&path).unwrap();
            let state = DemoState {
                node_id: "axi-node-4".into(),
                open_orders: 7,
                escrow_total_axi: 300,
            };
            store.save_snapshot("runtime_state", &state).unwrap();
        }

        let restarted = PersistentStore::open(&path).unwrap();
        let restored: Option<DemoState> = restarted.load_snapshot("runtime_state").unwrap();

        assert_eq!(
            restored,
            Some(DemoState {
                node_id: "axi-node-4".into(),
                open_orders: 7,
                escrow_total_axi: 300,
            })
        );

        let _ = fs::remove_file(path);
    }
}
