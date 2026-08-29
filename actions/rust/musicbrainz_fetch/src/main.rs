mod client;
mod core;
mod models;
mod writer;

use anyhow::{Context, Result};
use clap::Parser;
use models::{
    ExecutionPlan, OverwriteMode, TargetFlags, TargetUrl, parse_musicbrainz_url,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Fetch raw JSON responses from MusicBrainz")]
struct Cli {
    #[arg(value_name = "URL")]
    url_pos: Option<String>,

    #[arg(long = "url")]
    url_flag: Option<String>,

    #[arg(short = 'd', long = "dir", default_value = ".")]
    dir: PathBuf,

    #[arg(long = "release")]
    release: bool,

    #[arg(long = "release-group")]
    release_group: bool,

    #[arg(long = "all-releases")]
    all_releases: bool,

    #[arg(short = 'f', long = "force")]
    force: bool,
}

const fn resolve_target_flags(cli: &Cli, target: &TargetUrl) -> TargetFlags {
    let any_flag = cli.release || cli.release_group || cli.all_releases;
    if !any_flag {
        return match target {
            TargetUrl::Release(_) => TargetFlags {
                release: true,
                release_group: false,
                all_releases: false,
            },
            TargetUrl::ReleaseGroup(_) => TargetFlags {
                release: false,
                release_group: true,
                all_releases: false,
            },
        };
    }

    TargetFlags {
        release: cli.release,
        release_group: cli.release_group,
        all_releases: cli.all_releases,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let raw_url = cli
        .url_pos
        .as_deref()
        .or(cli.url_flag.as_deref())
        .context("Missing required MusicBrainz URL argument")?;

    let target = parse_musicbrainz_url(raw_url)
        .context("Invalid or unsupported MusicBrainz URL")?;

    let flags = resolve_target_flags(&cli, &target);

    let output_dir = if cli.dir.starts_with("~") {
        libactions::paths::expand_path(&cli.dir.to_string_lossy())
    } else {
        cli.dir
    };

    let overwrite = if cli.force {
        OverwriteMode::Force
    } else {
        OverwriteMode::Preserve
    };

    let plan = ExecutionPlan {
        target,
        output_dir,
        overwrite,
        flags,
    };

    core::execute(plan).await?;

    Ok(())
}
