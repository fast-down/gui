use std::{fs, path::Path};

#[tauri::command]
pub fn format_dir(dir: &str) -> Result<Option<String>, String> {
    let path = Path::new(dir).to_path_buf();
    if path.is_dir() {
        let path = fs::canonicalize(path).map_err(|e| format!("{e:?}"))?;
        Ok(Some(path.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}
