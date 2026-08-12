use crate::models::{DiscogsSearchResult, UserSelection};
use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn format_year(year_val: Option<&serde_json::Value>) -> String {
    year_val.map_or_else(
        || "N/A".to_string(),
        |v| match v {
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s.clone(),
            _ => "N/A".to_string(),
        },
    )
}

pub fn format_styles(genres: &[String], styles: &[String]) -> String {
    let mut combined = Vec::new();
    for g in genres {
        if !combined.contains(g) {
            combined.push(g.clone());
        }
    }
    for s in styles {
        if !combined.contains(s) {
            combined.push(s.clone());
        }
    }
    if combined.is_empty() {
        String::new()
    } else {
        format!(" [{}]", combined.join(", "))
    }
}

pub fn prompt_selection(
    header: &str,
    results: &[DiscogsSearchResult],
) -> Result<UserSelection> {
    println!("\n\x1b[1;36m{header}\x1b[0m");

    for (idx, res) in results.iter().enumerate() {
        let year_str = format_year(res.year.as_ref());
        let style_str = format_styles(&res.genre, &res.style);
        let id = res.id;
        let title = &res.title;
        let num = idx + 1;
        println!("   [{num}] {title} ({year_str}) — Master #{id}{style_str}");
    }

    let count = results.len();
    let prompt = format!("\nSelect Master Release [1-{count}] (Default: 1, s: Skip, q: Quit): ");

    let input = prompt_tty(&prompt)?;
    let trimmed = input.trim().to_lowercase();

    if trimmed.is_empty() || trimmed == "1" {
        return Ok(UserSelection::Selected(0));
    }
    if trimmed == "s" || trimmed == "skip" {
        return Ok(UserSelection::Skip);
    }
    if trimmed == "q" || trimmed == "quit" {
        return Ok(UserSelection::Quit);
    }

    if let Ok(num) = trimmed.parse::<usize>()
        && num >= 1
        && num <= count
    {
        return Ok(UserSelection::Selected(num - 1));
    }

    println!("\x1b[33mInvalid choice, defaulting to choice 1.\x1b[0m");
    Ok(UserSelection::Selected(0))
}

fn prompt_tty(prompt: &str) -> Result<String> {
    let mut tty_out = File::create("/dev/tty")?;
    tty_out.write_all(prompt.as_bytes())?;
    tty_out.flush()?;

    let tty_in = File::open("/dev/tty")?;
    let mut reader = BufReader::new(tty_in);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line)
}
