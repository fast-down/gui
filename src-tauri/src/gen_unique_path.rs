use crate::format_dir;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct UniquePath {
    pub dir: String,
    pub name: String,
    pub path: String,
}

#[tauri::command]
pub fn gen_unique_path(dir: &str, name: &str) -> Result<UniquePath, String> {
    let dir = format_dir::format_dir(dir)?.ok_or("No a directory")?;
    let path = Path::new(&dir).join(name);
    if !path.try_exists().map_err(|e| e.to_string())? {
        return Ok(UniquePath {
            dir,
            name: name.to_string(),
            path: path.to_string_lossy().to_string(),
        });
    }
    let path_obj = Path::new(name);
    let stem = path_obj.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let ext = path_obj
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| format!(".{s}"))
        .unwrap_or_default();
    for i in 1.. {
        let new_name = format!("{stem} ({i}){ext}");
        let new_path = Path::new(&dir).join(&new_name);
        if !new_path.try_exists().map_err(|e| e.to_string())? {
            return Ok(UniquePath {
                dir,
                name: new_name,
                path: new_path.to_string_lossy().to_string(),
            });
        }
    }
    unreachable!()
}
