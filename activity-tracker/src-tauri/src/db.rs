use rusqlite::{Connection, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Activity {
    pub id: i32,
    pub app_name: String,
    pub window_title: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AggregatedActivity {
    pub app_name: String,
    pub window_title: String,
    pub duration: i64, // in seconds
}

pub fn init_db(db_path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS activity (
            id INTEGER PRIMARY KEY,
            app_name TEXT NOT NULL,
            window_title TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ignored_apps (
            app_name TEXT PRIMARY KEY
        )",
        [],
    )?;
    Ok(conn)
}

pub fn insert_activity(conn: &Connection, activity: &Activity) -> Result<usize> {
    conn.execute(
        "INSERT INTO activity (app_name, window_title, start_time, end_time) VALUES (?1, ?2, ?3, ?4)",
        (&activity.app_name, &activity.window_title, &activity.start_time.to_rfc3339(), &activity.end_time.to_rfc3339()),
    )
}

pub fn get_daily_report(conn: &Connection, date: &str) -> Result<Vec<AggregatedActivity>> {
    let mut stmt = conn.prepare("SELECT app_name, window_title, SUM(strftime('%s', end_time) - strftime('%s', start_time)) as duration FROM activity WHERE date(start_time) = ?1 GROUP BY app_name, window_title")?;
    let activity_iter = stmt.query_map([date], |row| {
        Ok(AggregatedActivity {
            app_name: row.get(0)?,
            window_title: row.get(1)?,
            duration: row.get(2)?,
        })
    })?;

    let mut activities = Vec::new();
    for activity in activity_iter {
        activities.push(activity?);
    }
    Ok(activities)
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query_map([key], |row| row.get(0))?;
    if let Some(row) = rows.next() {
        row.map(Some)
    } else {
        Ok(None)
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<usize> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        [key, value],
    )
}

pub fn get_ignored_apps(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT app_name FROM ignored_apps")?;
    let mut rows = stmt.query_map([], |row| row.get(0))?;
    let mut apps = Vec::new();
    while let Some(app) = rows.next() {
        apps.push(app?);
    }
    Ok(apps)
}

pub fn add_ignored_app(conn: &Connection, app_name: &str) -> Result<usize> {
    conn.execute(
        "INSERT OR IGNORE INTO ignored_apps (app_name) VALUES (?1)",
        [app_name],
    )
}

pub fn remove_ignored_app(conn: &Connection, app_name: &str) -> Result<usize> {
    conn.execute(
        "DELETE FROM ignored_apps WHERE app_name = ?1",
        [app_name],
    )
}
