use serde::{Serialize, Deserialize};
use db::AggregatedActivity;
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
fn is_idle(state: tauri::State<Arc<Mutex<SystemTime>>>, conn: tauri::State<Arc<Mutex<rusqlite::Connection>>>) -> bool {
    let last_activity = *state.lock().unwrap();
    let now = SystemTime::now();
    let idle_time = get_idle_time(conn).unwrap_or(300);
    now.duration_since(last_activity).unwrap().as_secs() > idle_time
}

#[tauri::command]
fn get_daily_report(date: String, conn: tauri::State<Arc<Mutex<rusqlite::Connection>>>) -> Result<Vec<AggregatedActivity>, String> {
    let conn = conn.lock().unwrap();
    db::get_daily_report(&conn, &date).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_idle_time(conn: tauri::State<Arc<Mutex<rusqlite::Connection>>>) -> Result<u64, String> {
    let conn = conn.lock().unwrap();
    let idle_time_str = db::get_setting(&conn, "idle_time").map_err(|e| e.to_string())?.unwrap_or("300".to_string());
    idle_time_str.parse::<u64>().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_idle_time(idle_time: u64, conn: tauri::State<Arc<Mutex<rusqlite::Connection>>>) -> Result<(), String> {
    let conn = conn.lock().unwrap();
    db::set_setting(&conn, "idle_time", &idle_time.to_string()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_ignored_apps(conn: tauri::State<Arc<Mutex<rusqlite::Connection>>>) -> Result<Vec<String>, String> {
    let conn = conn.lock().unwrap();
    db::get_ignored_apps(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_ignored_app(app_name: String, conn: tauri::State<Arc<Mutex<rusqlite::Connection>>>) -> Result<(), String> {
    let conn = conn.lock().unwrap();
    db::add_ignored_app(&conn, &app_name).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn remove_ignored_app(app_name: String, conn: tauri::State<Arc<Mutex<rusqlite::Connection>>>) -> Result<(), String> {
    let conn = conn.lock().unwrap();
    db::remove_ignored_app(&conn, &app_name).map_err(|e| e.to_string())?;
    Ok(())
}

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            if !app_data_dir.exists() {
                std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data dir");
            }
            let db_path = app_data_dir.join("activity.db");

            let conn = db::init_db(&db_path).expect("Failed to initialize database");
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
                            let idle_time = db::get_setting(&conn, "idle_time").unwrap().unwrap_or("300".to_string()).parse::<u64>().unwrap();
                            let is_idle = now.duration_since(*last_activity).unwrap_or_default().as_secs() > idle_time;
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
                                        let ignored_apps = db::get_ignored_apps(&conn).unwrap_or_default();
                                        if ignored_apps.contains(&info.app_name) {
                                            continue;
                                        }
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

            app.manage(conn);
            app.manage(last_activity);
            app.manage(current_activity);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, get_active_window_info, is_idle, get_daily_report, get_idle_time, set_idle_time, get_ignored_apps, add_ignored_app, remove_ignored_app])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
