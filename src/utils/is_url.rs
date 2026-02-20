use url::Url;

pub fn is_url(s: &str) -> bool {
    Url::parse(s).is_ok_and(|u| ["http", "https"].contains(&u.scheme()))
}
