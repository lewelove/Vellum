use regex::Regex;
use std::sync::LazyLock;

pub static INTERPOLATION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\$\{([^}]+)\}").unwrap()
});

pub static PATTERN_LITERAL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"%\{([^}]+)\}").unwrap()
});

pub static STANDARD_LITERAL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\{([^}]+)\}").unwrap()
});

fn sql_quote(s: &str) -> String {
    s.replace('\'', "''")
}

pub fn expand_shorthand(input: &str) -> String {
    let mut res = input.to_string();
    
    res = INTERPOLATION_RE.replace_all(&res, |caps: &regex::Captures| {
        let inner = &caps[1];
        let path = if inner.starts_with("$.") {
            inner.to_string()
        } else {
            format!("$.{inner}")
        };
        format!("json_extract(metadata, '{path}')")
    }).to_string();

    res = PATTERN_LITERAL_RE.replace_all(&res, |caps: &regex::Captures| {
        format!("'%{}%'", sql_quote(&caps[1]))
    }).to_string();

    res = STANDARD_LITERAL_RE.replace_all(&res, |caps: &regex::Captures| {
        format!("'{}'", sql_quote(&caps[1]))
    }).to_string();

    res
}

