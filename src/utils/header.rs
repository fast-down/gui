use std::collections::HashMap;

pub fn parse_header(input: &str) -> impl Iterator<Item = (&str, &str)> {
    input
        .lines()
        .map(|s| s.splitn(2, ":").map(|s| s.trim()))
        .map(|mut iter| (iter.next(), iter.next()))
        .filter_map(|e| match e {
            (None, None) => None,
            (None, Some(_)) => None,
            (Some(key), None) => Some((key, "")),
            (Some(key), Some(value)) => Some((key, value)),
        })
}

pub fn parse_header_hashmap(input: &str) -> HashMap<String, String> {
    parse_header(input)
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}
