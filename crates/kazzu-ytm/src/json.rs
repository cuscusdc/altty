//! Small helpers to walk innertube JSON without a typed schema.
//!
//! Innertube responses are deeply nested and change shape often, so parsers
//! address fields by path and degrade to `None` instead of failing.

use serde_json::Value;

/// Follows a path of object keys and array indices (`"0"`, `"1"`, ...).
pub(crate) fn at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cur = value;
    for seg in path {
        cur = match cur {
            Value::Object(map) => map.get(*seg)?,
            Value::Array(items) => items.get(seg.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(cur)
}

pub(crate) fn str_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    at(value, path)?.as_str()
}

pub(crate) fn arr_at<'a>(value: &'a Value, path: &[&str]) -> &'a [Value] {
    at(value, path)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

/// Text of a `{runs: [...]}` or `{simpleText: ...}` object.
pub(crate) fn text(value: &Value) -> Option<String> {
    if let Some(s) = value.get("simpleText").and_then(Value::as_str) {
        return Some(s.to_owned());
    }
    let runs = value.get("runs")?.as_array()?;
    let s: String = runs
        .iter()
        .filter_map(|r| r.get("text").and_then(Value::as_str))
        .collect();
    (!s.is_empty()).then_some(s)
}

pub(crate) fn text_at(value: &Value, path: &[&str]) -> Option<String> {
    at(value, path).and_then(text)
}

/// The `runs` array of a text object (empty when absent).
pub(crate) fn runs(value: Option<&Value>) -> &[Value] {
    value
        .and_then(|v| v.get("runs"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

/// Every value stored under `key`, anywhere below `value`, in document
/// order. Does not descend into a match (so nested renderers of the same
/// type are not reported twice).
pub(crate) fn find_all<'a>(value: &'a Value, key: &str) -> Vec<&'a Value> {
    let mut out = Vec::new();
    find_into(value, key, &mut out);
    out
}

fn find_into<'a>(value: &'a Value, key: &str, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                if k == key {
                    out.push(v);
                } else {
                    find_into(v, key, out);
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|v| find_into(v, key, out)),
        _ => {}
    }
}

/// First value stored under `key`, anywhere below `value`.
pub(crate) fn find_first<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    match value {
        Value::Object(map) => {
            if let Some(v) = map.get(key) {
                return Some(v);
            }
            map.values().find_map(|v| find_first(v, key))
        }
        Value::Array(items) => items.iter().find_map(|v| find_first(v, key)),
        _ => None,
    }
}

/// All string leaves below `value` (for tolerant scans).
pub(crate) fn strings(value: &Value) -> Vec<&str> {
    let mut out = Vec::new();
    fn go<'a>(v: &'a Value, out: &mut Vec<&'a str>) {
        match v {
            Value::String(s) => out.push(s),
            Value::Array(a) => a.iter().for_each(|x| go(x, out)),
            Value::Object(m) => m.values().for_each(|x| go(x, out)),
            _ => {}
        }
    }
    go(value, &mut out);
    out
}

/// Parses `m:ss` / `h:mm:ss` into seconds.
pub(crate) fn parse_duration(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() || !s.contains(':') {
        return None;
    }
    let mut total = 0u64;
    for part in s.split(':') {
        if part.is_empty() || part.len() > 2 || !part.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        total = total * 60 + part.parse::<u64>().ok()?;
    }
    Some(total)
}

/// A 4-digit year between 1900 and 2100.
pub(crate) fn parse_year(s: &str) -> Option<i32> {
    let s = s.trim();
    (s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()))
        .then(|| s.parse().ok())
        .flatten()
        .filter(|y| (1900..=2100).contains(y))
}

/// Number stored either as JSON number or as a decimal string.
pub(crate) fn num(value: Option<&Value>) -> Option<u64> {
    match value? {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn path_and_text() {
        let v = json!({"a": [{"b": {"runs": [{"text": "x"}, {"text": "y"}]}}]});
        assert_eq!(text_at(&v, &["a", "0", "b"]).as_deref(), Some("xy"));
        assert!(at(&v, &["a", "1"]).is_none());
        assert_eq!(text(&json!({"simpleText": "s"})).as_deref(), Some("s"));
        assert_eq!(find_all(&v, "text").len(), 2);
    }

    #[test]
    fn durations_and_years() {
        assert_eq!(parse_duration("4:38"), Some(278));
        assert_eq!(parse_duration("1:02:03"), Some(3723));
        assert_eq!(parse_duration("abc"), None);
        assert_eq!(parse_duration("2024"), None);
        assert_eq!(parse_year("1991"), Some(1991));
        assert_eq!(parse_year("12345"), None);
    }
}
