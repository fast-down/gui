use std::path::{Component, Path, PathBuf};

pub fn sanitize(filename: impl AsRef<str>, max_units: usize) -> String {
    let filename = filename.as_ref();
    let options = sanitize_filename::Options {
        windows: cfg!(windows),
        truncate: false,
        replacement: "_",
    };
    let cleaned = sanitize_filename::sanitize_with_options(filename, options);
    let (base, ext) = if let Some(pos) = cleaned.rfind('.') {
        let ext_candidate = &cleaned[pos..];
        // 如果点后面的内容太长（>16字节），视作文件名的一部分而非扩展名
        if ext_candidate.len() > 16 {
            (&cleaned[..], "")
        } else {
            (&cleaned[..pos], ext_candidate)
        }
    } else {
        (&cleaned[..], "")
    };
    let final_base = truncate_filename(base, ext, max_units);
    format!("{}{}", final_base, ext)
}

pub fn sanitize_path(path: &Path) -> PathBuf {
    path.components()
        .map(|c| match c {
            Component::Normal(name) => {
                let s = name.to_string_lossy();
                sanitize(&s, 255).into()
            }
            Component::ParentDir => "..".into(),
            Component::CurDir => ".".into(),
            Component::RootDir => std::path::MAIN_SEPARATOR_STR.into(),
            Component::Prefix(prefix) => prefix.as_os_str().to_os_string(),
        })
        .collect()
}

#[cfg(windows)]
fn truncate_filename<'a>(base: &'a str, ext: &str, max_units: usize) -> &'a str {
    let ext_units = ext.encode_utf16().count();
    let max_base_units = max_units.saturating_sub(ext_units);
    let mut current_units = 0;
    let mut byte_index = 0;
    for (i, c) in base.char_indices() {
        let units = c.len_utf16();
        if current_units + units > max_base_units {
            break;
        }
        current_units += units;
        byte_index = i + c.len_utf8();
    }
    &base[..byte_index]
}

#[cfg(unix)]
fn truncate_filename<'a>(base: &'a str, ext: &str, max_units: usize) -> &'a str {
    let ext_bytes = ext.len();
    let max_base_bytes = max_units.saturating_sub(ext_bytes);
    if base.len() <= max_base_bytes {
        return base;
    }
    let mut end = max_base_bytes;
    while end > 0 && !base.is_char_boundary(end) {
        end -= 1;
    }
    &base[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    fn assert_length(s: &str, max_units: usize) {
        let units = s.encode_utf16().count();
        assert!(
            units <= max_units,
            "Windows: length {} exceeded {}",
            units,
            max_units
        );
    }
    #[cfg(unix)]
    fn assert_length(s: &str, max_units: usize) {
        let units = s.len();
        assert!(
            units <= max_units,
            "Unix: length {} exceeded {}",
            units,
            max_units
        );
    }

    #[test]
    fn test_sanitize() {
        // 文件名：file_stem.ext
        // 测试长文件名保留后缀（当 ext 较短时优先截断 file_stem）
        let long_stem = "这是一个非常".repeat(50);
        let long_name = format!("{}.mp4", long_stem);
        let result = sanitize(&long_name, 255);
        assert!(result.ends_with(".mp4"));
        assert_length(&result, 255);

        // 文件名：file_stem.ext
        // 测试非常长的后缀名（当 ext 过长时，可能他并没有扩展名，类似“1.这是第一个标题”，显然“这是第一个标题”不是文件后缀名，因此 file_stem.ext 当成整个文件名截断
        let long_stem = "这是一个非常".repeat(50);
        let long_name = format!("1.{}", long_stem);
        let result = sanitize(&long_name, 255);
        assert_length(&result, 255);

        // 测试普通后缀
        let normal_name = "我的文件.test.txt";
        let result = sanitize(normal_name, 255);
        assert_eq!(result, "我的文件.test.txt");

        // 测试包含非法字符
        let illegal = "test/\\:*?\"<>|.png";
        let result = sanitize(illegal, 255);
        assert_eq!(result, "test_________.png");
    }
}
