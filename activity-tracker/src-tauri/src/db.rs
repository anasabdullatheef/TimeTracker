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

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("activity.db")?;
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
    Ok(conn)
}

pub fn insert_activity(conn: &Connection, activity: &Activity) -> Result<usize> {
    conn.execute(
        "INSERT INTO activity (app_name, window_title, start_time, end_time) VALUES (?1, ?2, ?3, ?4)",
        (&activity.app_name, &activity.window_title, &activity.start_time.to_rfc3339(), &activity.end_time.to_rfc3339()),
    )
}

pub fn get_daily_report(conn: &Connection, date: &str) -> Result<Vec<Activity>> {
    let mut stmt = conn.prepare("SELECT id, app_name, window_title, start_time, end_time FROM activity WHERE date(start_time) = ?1")?;
    let activity_iter = stmt.query_map([date], |row| {
        Ok(Activity {
            id: row.get(0)?,
            app_name: row.get(1)?,
            window_title: row.get(2)?,
            start_time: row.get::<_, String>(3)?.parse::<DateTime<Utc>>().unwrap(),
            end_time: row.get::<_, String>(4)?.parse::<DateTime<Utc>>().unwrap(),
        })
    })?;

    let mut activities = Vec::new();
    for activity in activity_iter {
        activities.push(activity?);
    }
    Ok(activities)
}
