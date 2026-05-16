use crate::config;
use rusqlite::{params, Connection, Result};
pub struct Database {
    conn: Connection,
}
impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open(config::DB_FILENAME)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS investigations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                ip TEXT NOT NULL,
                process_name TEXT,
                risk_score INTEGER,
                country TEXT,
                isp TEXT,
                whois_data TEXT
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS firewall_blocks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                ip TEXT UNIQUE NOT NULL,
                process_name TEXT,
                rule_name TEXT NOT NULL
            )",
            [],
        )?;
        Ok(Database { conn })
    }
    pub fn save_investigation(
        &self,
        ip: &str,
        name: &str,
        score: u8,
        country: &str,
        isp: &str,
        whois: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO investigations (ip, process_name, risk_score, country, isp, whois_data) VALUES (?, ?, ?, ?, ?, ?)",
            params![ip, name, score, country, isp, whois],
        )?;
        Ok(())
    }
    pub fn add_firewall_block(&self, ip: &str, name: &str, rule: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO firewall_blocks (ip, process_name, rule_name) VALUES (?, ?, ?)",
            params![ip, name, rule],
        )?;
        Ok(())
    }
    pub fn remove_firewall_block(&self, ip: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM firewall_blocks WHERE ip = ?", params![ip])?;
        Ok(())
    }
    pub fn get_blocked_ips(&self) -> Result<Vec<(String, String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT ip, process_name, rule_name FROM firewall_blocks")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}
