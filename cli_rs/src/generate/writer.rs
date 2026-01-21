use crate::config::{LayoutConfig, LayoutItem};
use std::collections::HashMap;
use super::compressor::get_layout_keys; // Changed to super::compressor

fn format_val(val: &str) -> String {
    serde_json::to_string(val).unwrap_or_else(|_| format!("\"{}\"", val))
}

pub fn render_toml_block(
    pool: &HashMap<String, String>,
    layout: Option<&LayoutConfig>
) -> Vec<String> {
    let mut lines = Vec::new();

    let explicit_keys = get_layout_keys(layout);
    let mut appendix_keys: Vec<_> = pool.keys()
        .filter(|k| !explicit_keys.contains(*k))
        .collect();
    appendix_keys.sort();

    let mut appendix_consumed = false;

    if let Some(cfg) = layout {
        for item in &cfg.layout {
            match item {
                LayoutItem::Key(s) => {
                    if s == "\n" {
                        lines.push(String::new());
                    } else if s == "*" {
                        if !appendix_consumed {
                            for k in &appendix_keys {
                                if let Some(v) = pool.get(*k) {
                                    lines.push(format!("{} = {}", k, format_val(v)));
                                }
                            }
                            appendix_consumed = true;
                        }
                    } else if s.starts_with("#") {
                         lines.push(s.clone());
                    } else if let Some(val) = pool.get(s) {
                        lines.push(format!("{} = {}", s, format_val(val)));
                    }
                },
                LayoutItem::Block(map) => {
                    for (header, tags) in map {
                        let mut has_content = false;
                        for t in tags {
                            if t == "*" && !appendix_consumed && !appendix_keys.is_empty() {
                                has_content = true;
                            } else if t != "\n" && pool.contains_key(t) {
                                has_content = true;
                            }
                        }

                        if has_content {
                            if !header.is_empty() {
                                lines.push(header.clone());
                            }
                            for t in tags {
                                if t == "\n" {
                                    lines.push(String::new());
                                } else if t == "*" {
                                    if !appendix_consumed {
                                        for k in &appendix_keys {
                                            if let Some(v) = pool.get(*k) {
                                                lines.push(format!("{} = {}", k, format_val(v)));
                                            }
                                        }
                                        appendix_consumed = true;
                                    }
                                } else if let Some(val) = pool.get(t) {
                                    lines.push(format!("{} = {}", t, format_val(val)));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if !appendix_consumed {
        for k in &appendix_keys {
            if let Some(v) = pool.get(*k) {
                lines.push(format!("{} = {}", k, format_val(v)));
            }
        }
    }

    lines
}
