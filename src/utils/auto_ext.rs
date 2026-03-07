use std::borrow::Cow;

/// 校验提取出来的字符是否像一个真实的扩展名
fn is_extension(ext: &str) -> bool {
    !ext.is_empty() && ext.len() <= 16 && ext.chars().all(|c| c.is_ascii_graphic())
}

fn best_ext<'a>(extensions: &[&'a str]) -> Option<&'a str> {
    extensions
        .iter()
        .copied()
        .min_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)))
}

pub fn auto_ext<'a>(file_name: &'a str, content_type: Option<&str>) -> Cow<'a, str> {
    let file_name = file_name.trim_end_matches('.');
    let has_valid_ext = file_name
        .rfind('.')
        .is_some_and(|pos| is_extension(&file_name[pos + 1..]));
    if has_valid_ext {
        return Cow::Borrowed(file_name);
    }
    if let Some(ct) = content_type
        && let Some(mime_type) = ct.split(';').next().map(|s| s.trim())
        && let Some(extensions) = mime_guess::get_mime_extensions_str(mime_type)
        && let Some(ext) = best_ext(extensions)
    {
        return Cow::Owned(format!("{}.{}", file_name, ext));
    }
    Cow::Borrowed(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_ext() {
        // 测试用例 1：已经有后缀名，原样返回
        let res = auto_ext("document.pdf", Some("application/pdf"));
        assert_eq!("document.pdf", res);

        // 测试用例 2：没有后缀名，正常解析 MIME
        let res = auto_ext("avatar", Some("image/jpeg"));
        assert_eq!("avatar.jpe", res);

        // 测试用例 3：Content-Type 带 charset 等附加参数
        let res = auto_ext("index", Some("text/html; charset=utf-8"));
        assert_eq!("index.htm", res);

        // 测试用例 4：没有后缀名，且 content_type 为 None
        let res = auto_ext("unknown_file", None);
        assert_eq!("unknown_file", res);

        // 测试用例 5：不合法的 Content-Type
        let res = auto_ext("some_file", Some("not-a-valid-mime"));
        assert_eq!("some_file", res);

        // 测试用例 6：类似隐藏文件
        let res = auto_ext(".gitignore", Some("text/plain"));
        assert_eq!(".gitignore", res);

        // 测试用例 7：测试干扰文件名
        let res = auto_ext("1.这是一个视频", Some("video/mp4"));
        assert_eq!("1.这是一个视频.mp4", res);
    }
}
