mod config;

use eyre::Result;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Deserialize;
use std::{collections::HashMap, fmt::Write, path::Path};
use tracing::Level;
use tracing::*;
use warp::Filter;

use crate::config::Config;
use crate::config::Server;

static CONFIG: Lazy<Config> = Lazy::new(Config::open_or_create);

#[derive(Debug, Deserialize)]
struct Stats {
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

async fn read_json(path: impl AsRef<Path>) -> Result<Stats> {
    #[derive(Debug, Deserialize)]
    struct Stats_ {
        stats: Stats,
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

    let client = Client::new();
    let req = client
        .get(format!(
            "https://api.mojang.com/user/profiles/{}/names",
            uuid
        ))
        .send()
        .await?;
    let text: Vec<S> = req.json().await?;

    let recent_name = text
        .into_iter()
        .max_by_key(|x| x.changed_to_at.unwrap_or(0))
        .unwrap();

    Ok(recent_name.name)
}

async fn scrape_server<P: AsRef<Path>>(stats_path: P, server_name: &str) -> Result<String> {
    let mut output = String::new();
    info!(%server_name, "Starting scrape");
    for file in std::fs::read_dir(stats_path)? {
        let file = file?;
        if !file.file_type()?.is_file() {
            continue;
        }
        let file_path = file.path();
        let player_uuid = file_path
            .file_stem()
            .ok_or_else(|| eyre::eyre!("no file stem"))?
            .to_str()
            .expect("filename is a uuid so should be valid unicode.");
        let player_name = get_player_name(player_uuid).await?;
        debug!(uuid = %player_uuid, name = %player_name, "Scraping player");
        let json = read_json(&file.path()).await?;
        if let Some(ref dropped) = json.dropped {
            trace!("Scraping minecraft:dropped");
            writeln!(
                output,
                "# HELP minecraft_items_dropped minecraft-prometheus-exporter"
            )?;
            writeln!(output, "# TYPE minecraft_items_dropped counter")?;

            for (name, &value) in dropped {
                writeln!(
                    output,
                    "minecraft_items_dropped{{\
                         server=\"{}\", player=\"{}\", item=\"{}\"\
                         }} {}",
                    server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref crafted) = json.crafted {
            trace!("Scraping minecraft:crafted");
            writeln!(
                output,
                "# HELP minecraft_items_crafted minecraft-prometheus-exporter"
            )?;
            writeln!(output, "# TYPE minecraft_items_crafted counter")?;
            for (name, &value) in crafted {
                writeln!(
                    output,
                    "minecraft_items_crafted{{\
                         server=\"{}\", player=\"{}\", item=\"{}\"\
                         }} {}",
                    server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref killed) = json.killed {
            trace!("Scraping minecraft:killed");
            writeln!(
                output,
                "# HELP minecraft_entities_killed minecraft-prometheus-exporter"
            )?;
            writeln!(output, "# TYPE minecraft_entities_killed counter")?;
            for (name, &value) in killed {
                writeln!(
                    output,
                    "minecraft_entities_killed{{\
                         server=\"{}\", player=\"{}\", entity=\"{}\"\
                         }} {}",
                    server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref broken) = json.broken {
            trace!("Scraping minecraft:broken");
            writeln!(
                output,
                "# HELP minecraft_blocks_broken minecraft-prometheus-exporter"
            )?;
            writeln!(output, "# TYPE minecraft_blocks_broken counter")?;
            for (name, &value) in broken {
                writeln!(
                    output,
                    "minecraft_blocks_broken{{\
                         server=\"{}\", player=\"{}\", block=\"{}\"\
                         }} {}",
                    server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref used) = json.used {
            trace!("Scraping minecraft:used");
            writeln!(
                output,
                "# HELP minecraft_items_used minecraft-prometheus-exporter"
            )?;
            writeln!(output, "# TYPE minecraft_items_used counter")?;
            for (name, &value) in used {
                writeln!(
                    output,
                    "minecraft_items_used{{\
                         server=\"{}\", player=\"{}\", item=\"{}\"\
                         }} {}",
                    server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref mined) = json.mined {
            trace!("Scraping minecraft:mined");
            writeln!(
                output,
                "# HELP minecraft_blocks_mined minecraft-prometheus-exporter"
            )?;
            writeln!(output, "# TYPE minecraft_blocks_mined counter")?;
            for (name, &value) in mined {
                writeln!(
                    output,
                    "minecraft_blocks_mined{{\
                         server=\"{}\", player=\"{}\", block=\"{}\"\
                         }} {}",
                    server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref custom) = json.custom {
            trace!("Scraping minecraft:custom");
            writeln!(
                output,
                "# HELP minecraft_custom minecraft-prometheus-exporter"
            )?;
            writeln!(output, "# TYPE minecraft_custom counter")?;
            for (name, &value) in custom {
                writeln!(
                    output,
                    "minecraft_custom{{\
                         server=\"{}\", player=\"{}\", item=\"{}\"\
                         }} {}",
                    server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref picked_up) = json.picked_up {
            trace!("Scraping minecraft:picked_up");
            writeln!(
                output,
                "# HELP minecraft_items_picked_up minecraft-prometheus-exporter"
            )?;
            writeln!(output, "# TYPE minecraft_items_picked_up counter")?;
            for (name, &value) in picked_up {
                writeln!(
                    output,
                    "minecraft_items_picked_up{{\
                         server=\"{}\", player=\"{}\", item=\"{}\"\
                         }} {}",
                    server_name, player_name, name, value
                )?;
            }
        }
        if let Some(ref killed_by) = json.killed_by {
            trace!("Scraping minecraft:killed_by");
            writeln!(
                output,
                "# HELP minecraft_entities_killed_by minecraft-prometheus-exporter"
            )?;
            writeln!(output, "# TYPE minecraft_entities_killed_by counter")?;
            for (name, &value) in killed_by {
                writeln!(
                    output,
                    "minecraft_entities_killed_by{{\
                         server=\"{}\", player=\"{}\", entity=\"{}\"\
                         }} {}",
                    server_name, player_name, name, value
                )?;
            }
        }
    }
    Ok(output)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .pretty()
            .finish(),
    )?;

    Lazy::force(&CONFIG);

    let filter = warp::path("metrics").and_then(|| async move {
        if false {
            // XXX: Type inference hint, I kinda hate it.
            return Err(warp::reject());
        }
        let mut output = String::new();
        for Server {
            stats_root,
            server_name,
        } in &CONFIG.servers
        {
            output.push_str(
                scrape_server(stats_root, server_name.as_str())
                    .await
                    .map_err(|e| {
                        error!(error = ?e, "");
                        warp::reject::reject()
                    })?
                    .as_str(),
            );
        }

        Ok(output)
    });

    info!(port = %CONFIG.port, "Started listening.");
    warp::serve(filter).run(([127, 0, 0, 1], CONFIG.port)).await;
    Ok(())
}
