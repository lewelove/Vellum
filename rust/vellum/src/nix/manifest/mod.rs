use anyhow::Result;
use lava_torrent::torrent::v1::Torrent;
use std::fmt::Write;
use std::path::Path;

pub fn run(torrent_path_str: &str, tracks_filter: &str) -> Result<()> {
    let torrent_path = Path::new(torrent_path_str).canonicalize()?;
    
    let output = std::process::Command::new("nix")
        .args(["hash", "file", torrent_path.to_str().unwrap()])
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!("Failed to hash torrent file with nix");
    }
    
    let torrent_sha256 = String::from_utf8(output.stdout)?.trim().to_string();
    let torrent = Torrent::read_from_file(&torrent_path)
        .map_err(|e| anyhow::anyhow!("Failed to parse torrent: {e}"))?;
        
    let allowed_exts: Vec<String> = tracks_filter
        .split(',')
        .map(|s| format!(".{}", s.trim().to_lowercase()))
        .collect();
        
    let mut track_lines = Vec::new();
    
    if let Some(files) = &torrent.files {
        let mut track_no = 1;
        for f in files {
            let path_buf = f.path.clone();
            let path_str = path_buf.to_string_lossy();
            let ext = path_buf.extension().and_then(|e| e.to_str()).map(|e| format!(".{}", e.to_lowercase())).unwrap_or_default();
            
            if allowed_exts.contains(&ext) {
                track_lines.push(format!(
                    "    {{\n      file = \"{path_str}\";\n      metadata = {{\n        tracknumber = {track_no};\n        title = \"{}\";\n      }};\n    }}",
                    path_buf.file_stem().unwrap_or_default().to_string_lossy()
                ));
                track_no += 1;
            }
        }
    } else {
        let name_str = &torrent.name;
        let path = Path::new(name_str);
        let ext = path.extension().and_then(|e| e.to_str()).map(|e| format!(".{}", e.to_lowercase())).unwrap_or_default();
        if allowed_exts.contains(&ext) {
            track_lines.push(format!(
                "    {{\n      file = \"{name_str}\";\n      metadata = {{\n        tracknumber = 1;\n        title = \"{}\";\n      }};\n    }}",
                path.file_stem().unwrap_or_default().to_string_lossy()
            ));
        }
    }

    let torrent_file_name = torrent_path.file_name().unwrap_or_default().to_string_lossy();
    
    let mut out = String::new();
    out.push_str("{ vellix ? (import <nixpkgs> {}).lib }:\n");
    out.push_str("vellix.mkAlbum {\n");
    let _ = writeln!(out, "  pname = \"{}\";", torrent.name.replace(' ', "-").to_lowercase());
    out.push_str("  sourceTorrent = {\n");
    let _ = writeln!(out, "    file = ./{torrent_file_name};");
    let _ = writeln!(out, "    sha256 = \"{torrent_sha256}\";");
    out.push_str("  };\n\n");
    out.push_str("  album.metadata = {\n");
    let _ = writeln!(out, "    album = \"{}\";", torrent.name);
    let _ = writeln!(out, "    albumartist = \"Unknown Artist\";");
    out.push_str("    date = \"0000\";\n");
    out.push_str("    genre = [ \"Unknown\" ];\n");
    out.push_str("  };\n\n");
    out.push_str("  tracks = [\n");
    for line in track_lines {
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str("  ];\n");
    out.push_str("}\n");
    
    println!("{out}");
    Ok(())
}
