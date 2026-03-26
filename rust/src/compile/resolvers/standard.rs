use serde_json::{Value, json};

pub fn get_raw_value<'a>(source: &'a Value, key: &str, args: &str) -> Option<&'a Value> {
    let check = |k: &str| -> Option<&'a Value> {
        let v = source.get(k)?;
        match v {
            Value::Null => None,
            Value::String(s) if s.trim().is_empty() => None,
            Value::Array(a) if a.is_empty() => None,
            _ => Some(v),
        }
    };

    if let Some(v) = check(key) {
        return Some(v);
    }
    
    if !args.is_empty() {
        for fallback in args.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            if let Some(v) = check(fallback) {
                return Some(v);
            }
        }
    }
    None
}

pub fn resolve_generic_string(source: &Value, key: &str, args: &str, default: &str) -> Value {
    if let Some(v) = get_raw_value(source, key, args) {
        match v {
            Value::String(s) => json!(s.trim()),
            Value::Array(arr) => json!(arr.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join("; ")),
            _ => json!(v.to_string().replace('"', "").trim()),
        }
    } else {
        json!(default)
    }
}

pub fn resolve_generic_list(source: &Value, key: &str, args: &str) -> Value {
    if let Some(v) = get_raw_value(source, key, args) {
        match v {
            Value::Array(arr) => {
                let list: Vec<String> = arr.iter().filter_map(|x| x.as_str().map(|s| s.trim().to_string())).filter(|s| !s.is_empty()).collect();
                json!(list)
            }
            Value::String(s) => {
                let list: Vec<String> = s.split(';').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
                json!(list)
            }
            _ => json!([v.to_string().replace('"', "").trim()]),
        }
    } else {
        json!(Vec::<String>::new())
    }
}

pub fn resolve_generic_integer(source: &Value, key: &str, args: &str) -> Value {
    if let Some(v) = get_raw_value(source, key, args) {
        match v {
            Value::Number(n) => json!(n.as_i64().unwrap_or(0)),
            Value::String(s) => json!(s.trim().parse::<i64>().unwrap_or(0)),
            _ => json!(0),
        }
    } else {
        json!(0)
    }
}

pub fn resolve_generic_float(source: &Value, key: &str, args: &str) -> Value {
    if let Some(v) = get_raw_value(source, key, args) {
        match v {
            Value::Number(n) => json!(n.as_f64().unwrap_or(0.0)),
            Value::String(s) => json!(s.trim().parse::<f64>().unwrap_or(0.0)),
            _ => json!(0.0),
        }
    } else {
        json!(0.0)
    }
}

pub fn resolve_generic_bool(source: &Value, key: &str, args: &str) -> Value {
    if let Some(v) = get_raw_value(source, key, args) {
        match v {
            Value::Bool(b) => json!(*b),
            Value::String(s) => {
                let s = s.trim().to_lowercase();
                json!(s == "true" || s == "1" || s == "yes")
            }
            Value::Number(n) => json!(n.as_i64().unwrap_or(0) > 0),
            _ => json!(false),
        }
    } else {
        json!(false)
    }
}

pub fn resolve_generic_string_fallback(
    source: &Value,
    album_source: &Value,
    key: &str,
    album_key: &str,
    default: &str,
) -> Value {
    let check = |v: &Value| -> Option<String> {
        match v {
            Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
            Value::Array(a) if !a.is_empty() => Some(a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join("; ")),
            Value::Null => None,
            _ => Some(v.to_string().replace('"', "").trim().to_string()),
        }
    };

    if let Some(v) = source.get(key).and_then(check) {
        json!(v)
    } else if let Some(v) = album_source.get(album_key).and_then(check) {
        json!(v)
    } else {
        json!(default)
    }
}

pub fn format_ms(ms: u64) -> String {
    let s = (ms / 1000) % 60;
    let m = (ms / (1000 * 60)) % 60;
    let h = ms / (1000 * 60 * 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}
