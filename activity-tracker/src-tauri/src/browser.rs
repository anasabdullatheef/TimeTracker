use std::process::Command;

pub fn get_browser_url(app_name: &str) -> Option<String> {
    if app_name != "Google Chrome" && app_name != "Safari" {
        return None;
    }

    let script = format!(
        r#"
        tell application "{}"
            get URL of active tab of front window
        end tell
    "#,
        app_name
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
    } else {
        None
    }
}
