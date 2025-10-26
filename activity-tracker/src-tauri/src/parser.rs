pub fn parse_title(app_name: &str, window_title: &str) -> String {
    if window_title.is_empty() {
        return "Unknown Activity".to_string();
    }

    if app_name.contains("Code") || app_name.contains("code") {
        if let Some(file_name) = window_title.split(" - ").next() {
            return file_name.to_string();
        }
    }

    // Default to the original title if no specific parsing is done.
    window_title.to_string()
}
