use std::path::Path;

#[tauri::command]
pub fn format_dir(dir: &str) -> Result<Option<String>, String> {
    let mut path = Path::new(dir).to_path_buf();
    if path.is_dir() {
        if path.is_relative()
            && let Ok(self_path) = std::env::current_dir()
        {
            path = self_path.join(path);
        }
        path = path_clean::clean(path);
        Ok(Some(path.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}
