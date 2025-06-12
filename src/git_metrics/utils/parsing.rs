use std::collections::HashMap;

pub fn parse_param_i64(params: &HashMap<String, String>, key: &str, default: i64) -> i64 {
    params
        .get(key)
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(default)
}

pub fn parse_param_string<'a>(
    params: &'a HashMap<String, String>,
    key: &str,
    default: &'a String,
) -> &'a String {
    params.get(key).unwrap_or(default)
}
