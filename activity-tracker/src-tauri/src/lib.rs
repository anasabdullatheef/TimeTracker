use serde::{Serialize, Deserialize};
use db::Activity;
use x_win::get_active_window;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use rdev::{listen, Event};
use chrono::Utc;

mod db;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ActiveWindowInfo {
    title: String,
    app_name: String,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_active_window_info() -> Result<ActiveWindowInfo, String> {
    match get_active_window() {
        Ok(active_window) => Ok(ActiveWindowInfo {
            title: active_window.title,
            app_name: active_window.info.name,
        }),
        Err(_) => Err("Error getting active window".to_string()),
    }
}

#[tauri::command]
fn is_idle(state: tauri::State<Arc<Mutex<SystemTime>>>) -> bool {
    let last_activity = *state.lock().unwrap();
    let now = SystemTime::now();
    now.duration_since(last_activity).unwrap().as_secs() > 300
}

#[tauri::command]
fn get_daily_report(date: String, conn: tauri::State<Arc<Mutex<rusqlite::Connection>>>) -> Result<Vec<Activity>, String> {
    let conn = conn.lock().unwrap();
    db::get_daily_report(&conn, &date).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let conn = db::init_db().expect("Failed to initialize database");
    let conn = Arc::new(Mutex::new(conn));
    let last_activity = Arc::new(Mutex::new(SystemTime::now()));
    let current_activity: Arc<Mutex<Option<db::Activity>>> = Arc::new(Mutex::new(None));

    let last_activity_clone = last_activity.clone();
    thread::spawn(move || {
        let callback = move |_event: Event| {
            if let Ok(mut last_activity) = last_activity_clone.lock() {
                *last_activity = SystemTime::now();
            }
        };

        if let Err(error) = listen(callback) {
            println!("Error: {:?}", error)
        }
    });

    let conn_clone = conn.clone();
    let last_activity_clone2 = last_activity.clone();
    let current_activity_clone = current_activity.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            if let Ok(mut conn) = conn_clone.lock() {
                if let Ok(last_activity) = last_activity_clone2.lock() {
                    let now = SystemTime::now();
                    let is_idle = now.duration_since(*last_activity).unwrap_or_default().as_secs() > 300;
                    if let Ok(mut current_activity) = current_activity_clone.lock() {
                        if is_idle {
                            if let Some(mut activity) = current_activity.take() {
                                activity.end_time = Utc::now();
                                if let Err(e) = db::insert_activity(&mut conn, &activity) {
                                    println!("Error inserting activity: {:?}", e);
                                }
                            }
                        } else {
                            if let Ok(info) = get_active_window_info() {
                                if let Some(activity) = current_activity.as_mut() {
                                    if activity.app_name != info.app_name || activity.window_title != info.title {
                                        let mut activity_to_insert = activity.clone();
                                        activity_to_insert.end_time = Utc::now();
                                        if let Err(e) = db::insert_activity(&mut conn, &activity_to_insert) {
                                            println!("Error inserting activity: {:?}", e);
                                        }
                                        *current_activity = Some(db::Activity {
                                            id: 0,
                                            app_name: info.app_name,
                                            window_title: info.title,
                                            start_time: Utc::now(),
                                            end_time: Utc::now(),
                                        });
                                    }
                                } else {
                                    *current_activity = Some(db::Activity {
                                        id: 0,
                                        app_name: info.app_name,
                                        window_title: info.title,
                                        start_time: Utc::now(),
                                        end_time: Utc::now(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_active_window_info, is_idle, get_daily_report])
        .manage(conn)
        .manage(last_activity)
        .manage(current_activity)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
