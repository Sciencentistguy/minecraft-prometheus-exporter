use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;

use eyre::Result;
use eyre::WrapErr;
use serde::Deserialize;
use tracing::*;

use crate::config::*;
use crate::write_prometheus_blurb;

#[derive(Debug, Deserialize)]
struct PlayerStats {
    #[serde(rename = "minecraft:dropped")]
    dropped: Option<HashMap<String, usize>>,
    #[serde(rename = "minecraft:crafted")]
    crafted: Option<HashMap<String, usize>>,
    #[serde(rename = "minecraft:killed")]
    killed: Option<HashMap<String, usize>>,
    #[serde(rename = "minecraft:broken")]
    broken: Option<HashMap<String, usize>>,
    #[serde(rename = "minecraft:used")]
    used: Option<HashMap<String, usize>>,
    #[serde(rename = "minecraft:mined")]
    mined: Option<HashMap<String, usize>>,
    #[serde(rename = "minecraft:custom")]
    custom: Option<HashMap<String, usize>>,
    #[serde(rename = "minecraft:picked_up")]
    picked_up: Option<HashMap<String, usize>>,
    #[serde(rename = "minecraft:killed_by")]
    killed_by: Option<HashMap<String, usize>>,
}

async fn get_player_stats(path: impl AsRef<Path>) -> Result<PlayerStats> {
    #[derive(Debug, Deserialize)]
    struct Stats_ {
        stats: PlayerStats,
        #[serde(rename = "DataVersion")]
        data_version: u64,
    }
    let file = tokio::fs::read_to_string(path.as_ref()).await?;
    let json: Stats_ = serde_json::from_str(file.as_str())?;
    Ok(json.stats)
}

async fn get_player_name(uuid: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct S {
        name: String,
        changed_to_at: Option<u64>,
    }

    let req = reqwest::get(format!(
        "https://api.mojang.com/user/profiles/{}/names",
        uuid
    ))
    .await?;
    let text: Vec<S> = req.json().await?;

    let recent_name = text
        .into_iter()
        .max_by_key(|x| x.changed_to_at.unwrap_or(0))
        .unwrap();

    Ok(recent_name.name)
}

pub async fn scrape_player_stats<W: Write>(server: &Server, writer: &mut W) -> Result<()> {
    info!(?server, "Scraping player stats");
    for file in std::fs::read_dir(&server.stats_root).wrap_err("Could not read stats_path")? {
        let file = file?;
        if !file.file_type()?.is_file() {
            continue;
        }
        let file_path = file.path();
        let player_uuid = file_path
            .file_stem()
            .and_then(|uuid| uuid.to_str())
            .ok_or_else(|| eyre::eyre!("File `{:?}` does not have a valid uuid", file_path))?;
        let player_name = get_player_name(player_uuid).await?;
        debug!(uuid = %player_uuid, name = %player_name, "Scraping player");
        let json = get_player_stats(&file.path()).await?;
        if let Some(ref dropped) = json.dropped {
            trace!("Scraping minecraft:dropped");
            write_prometheus_blurb!(writer, "minecraft_items_dropped", "counter");
            for (name, &value) in dropped {
                writeln!(
                    writer,
                    "minecraft_items_dropped{{\
                         server=\"{}\", player=\"{}\", item=\"{}\"\
                         }} {}",
                    server.server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref crafted) = json.crafted {
            trace!("Scraping minecraft:crafted");
            write_prometheus_blurb!(writer, "minecraft_items_crafted", "counter");
            for (name, &value) in crafted {
                writeln!(
                    writer,
                    "minecraft_items_crafted{{\
                         server=\"{}\", player=\"{}\", item=\"{}\"\
                         }} {}",
                    server.server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref killed) = json.killed {
            trace!("Scraping minecraft:killed");
            write_prometheus_blurb!(writer, "minecraft_entities_killed", "counter");
            for (name, &value) in killed {
                writeln!(
                    writer,
                    "minecraft_entities_killed{{\
                         server=\"{}\", player=\"{}\", entity=\"{}\"\
                         }} {}",
                    server.server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref broken) = json.broken {
            trace!("Scraping minecraft:broken");
            write_prometheus_blurb!(writer, "minecraft_blocks_broken", "counter");
            for (name, &value) in broken {
                writeln!(
                    writer,
                    "minecraft_blocks_broken{{\
                         server=\"{}\", player=\"{}\", block=\"{}\"\
                         }} {}",
                    server.server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref used) = json.used {
            trace!("Scraping minecraft:used");
            write_prometheus_blurb!(writer, "minecraft_items_used", "counter");
            for (name, &value) in used {
                writeln!(
                    writer,
                    "minecraft_items_used{{\
                         server=\"{}\", player=\"{}\", item=\"{}\"\
                         }} {}",
                    server.server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref mined) = json.mined {
            trace!("Scraping minecraft:mined");
            write_prometheus_blurb!(writer, "minecraft_blocks_mined", "counter");
            for (name, &value) in mined {
                writeln!(
                    writer,
                    "minecraft_blocks_mined{{\
                         server=\"{}\", player=\"{}\", block=\"{}\"\
                         }} {}",
                    server.server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref custom) = json.custom {
            trace!("Scraping minecraft:custom");
            write_prometheus_blurb!(writer, "minecraft_custom", "gauge");
            for (name, &value) in custom {
                writeln!(
                    writer,
                    "minecraft_custom{{\
                         server=\"{}\", player=\"{}\", item=\"{}\"\
                         }} {}",
                    server.server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref picked_up) = json.picked_up {
            trace!("Scraping minecraft:picked_up");
            write_prometheus_blurb!(writer, "minecraft_items_picked_up", "counter");
            for (name, &value) in picked_up {
                writeln!(
                    writer,
                    "minecraft_items_picked_up{{\
                         server=\"{}\", player=\"{}\", item=\"{}\"\
                         }} {}",
                    server.server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref killed_by) = json.killed_by {
            trace!("Scraping minecraft:killed_by");
            write_prometheus_blurb!(writer, "minecraft_entities_killed_by", "counter");
            for (name, &value) in killed_by {
                writeln!(
                    writer,
                    "minecraft_entities_killed_by{{\
                         server=\"{}\", player=\"{}\", entity=\"{}\"\
                         }} {}",
                    server.server_name, player_name, name, value
                )?;
            }
        }
    }

    Ok(())
}
