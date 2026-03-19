use indexmap::IndexMap;
use serde_json::Value;
use std::collections::HashSet;

fn format_toml_value(val: &Value) -> String {
    match val {
        Value::String(s) => serde_json::to_string(s).unwrap_or_default(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(arr) => {
            let formatted: Vec<String> = arr.iter().map(format_toml_value).collect();
            format!("[{}]", formatted.join(", "))
        }
        _ => serde_json::to_string(val).unwrap_or_default(),
    }
}

pub fn render_toml_block(
    pool: &serde_json::Map<String, Value>,
    layout: Option<&IndexMap<String, toml::Value>>,
    level: &str,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut layout_keys = HashSet::new();

    if let Some(lay) = layout {
        for key in lay.keys() {
            layout_keys.insert(key.to_uppercase());
        }

        for (key, meta) in lay {
            if let Some(meta_table) = meta.as_table() {
                let key_level = meta_table.get("level").and_then(|v| v.as_str()).unwrap_or("");
                if key_level == level {
                    let val = pool.get(key).or_else(|| pool.get(&key.to_uppercase()));
                    let rendered_val = match val {
                        Some(v) => format_toml_value(v),
                        None => "\"\"".to_string(),
                    };
                    lines.push(format!("{} = {rendered_val}", key.to_uppercase()));

                    let add_newline = meta_table
                        .get("add_newline")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if add_newline {
                        lines.push(String::new());
                    }
                }
            }
        }
    }

    let mut appendix_keys: Vec<String> = pool
        .keys()
        .filter(|k| !layout_keys.contains(&k.to_uppercase()))
        .cloned()
        .collect();
    appendix_keys.sort();

    for k in appendix_keys {
        if let Some(v) = pool.get(&k) {
            lines.push(format!("{} = {}", k.to_uppercase(), format_toml_value(v)));
        }
    }

    lines
}
