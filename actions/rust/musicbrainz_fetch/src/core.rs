use crate::client::MusicBrainzClient;
use crate::models::{ExecutionPlan, OverwriteMode, TargetDemand, TargetFlags, TargetUrl};
use crate::writer::write_json_atomic;
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

const RELEASE_FILENAME: &str = "musicbrainz_release.json";
const RELEASE_GROUP_FILENAME: &str = "musicbrainz_releasegroup.json";
const ALL_RELEASES_FILENAME: &str = "musicbrainz_all_releases.json";

struct ResolvedPaths {
    release: PathBuf,
    release_group: PathBuf,
    all_releases: PathBuf,
}

pub async fn execute(plan: ExecutionPlan) -> Result<()> {
    let paths = ResolvedPaths {
        release: plan.output_dir.join(RELEASE_FILENAME),
        release_group: plan.output_dir.join(RELEASE_GROUP_FILENAME),
        all_releases: plan.output_dir.join(ALL_RELEASES_FILENAME),
    };

    let client = MusicBrainzClient::new()?;

    match plan.target {
        TargetUrl::Release(ref release_id) => {
            execute_release_flow(&client, release_id, &paths, plan.flags, plan.overwrite)
                .await
        }
        TargetUrl::ReleaseGroup(ref rg_id) => {
            execute_release_group_flow(&client, rg_id, &paths, plan.flags, plan.overwrite)
                .await
        }
    }
}

async fn execute_release_flow(
    client: &MusicBrainzClient,
    release_id: &str,
    paths: &ResolvedPaths,
    flags: TargetFlags,
    overwrite: OverwriteMode,
) -> Result<()> {
    let need_rel = should_write(&paths.release, flags.release.into(), overwrite);
    let need_rg =
        should_write(&paths.release_group, flags.release_group.into(), overwrite);
    let need_all =
        should_write(&paths.all_releases, flags.all_releases.into(), overwrite);

    if !need_rel && !need_rg && !need_all {
        return Ok(());
    }

    let rel_val = client.fetch_release(release_id).await?;

    if need_rel {
        write_json_atomic(&paths.release, &rel_val)?;
        println!(
            "\x1b[32m✔\x1b[0m Saved release to: {}",
            paths.release.display()
        );
    }

    if need_rg || need_all {
        let rg_id = rel_val
            .get("release-group")
            .and_then(|rg| rg.get("id"))
            .and_then(Value::as_str)
            .context("Release response missing release-group ID")?
            .to_string();

        if need_rg {
            MusicBrainzClient::wait_rate_limit().await;
            let rg_val = client.fetch_release_group(&rg_id).await?;
            write_json_atomic(&paths.release_group, &rg_val)?;
            println!(
                "\x1b[32m✔\x1b[0m Saved release-group to: {}",
                paths.release_group.display()
            );
        }

        if need_all {
            MusicBrainzClient::wait_rate_limit().await;
            let all_val = client.browse_all_releases(&rg_id).await?;
            write_json_atomic(&paths.all_releases, &all_val)?;
            println!(
                "\x1b[32m✔\x1b[0m Saved all releases to: {}",
                paths.all_releases.display()
            );
        }
    }

    Ok(())
}

async fn execute_release_group_flow(
    client: &MusicBrainzClient,
    rg_id: &str,
    paths: &ResolvedPaths,
    flags: TargetFlags,
    overwrite: OverwriteMode,
) -> Result<()> {
    let need_rg =
        should_write(&paths.release_group, flags.release_group.into(), overwrite);
    let need_all =
        should_write(&paths.all_releases, flags.all_releases.into(), overwrite);

    if flags.release && !flags.release_group && !flags.all_releases {
        eprintln!("Error: Cannot derive release from release-group URL");
        std::process::exit(1);
    }

    if flags.release {
        eprintln!("Notice: Skipping release target for release-group URL");
    }

    if !need_rg && !need_all {
        return Ok(());
    }

    let fetched_rg = if need_rg {
        let rg_val = client.fetch_release_group(rg_id).await?;
        write_json_atomic(&paths.release_group, &rg_val)?;
        println!(
            "\x1b[32m✔\x1b[0m Saved release-group to: {}",
            paths.release_group.display()
        );
        true
    } else {
        false
    };

    if need_all {
        if fetched_rg {
            MusicBrainzClient::wait_rate_limit().await;
        }
        let all_val = client.browse_all_releases(rg_id).await?;
        write_json_atomic(&paths.all_releases, &all_val)?;
        println!(
            "\x1b[32m✔\x1b[0m Saved all releases to: {}",
            paths.all_releases.display()
        );
    }

    Ok(())
}

fn should_write(path: &Path, demand: TargetDemand, overwrite: OverwriteMode) -> bool {
    if demand == TargetDemand::Omitted {
        return false;
    }
    if overwrite == OverwriteMode::Force {
        return true;
    }
    !path.exists()
}
