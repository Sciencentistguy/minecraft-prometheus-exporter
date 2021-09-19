use std::fmt::Write;

use eyre::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use tracing::*;

use crate::config::Server;

static LIST_RESPONSE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"There are (\d+) of a max of (\d+) players online: ?(.*)?$").unwrap());

#[derive(Debug)]
struct RconPlayersData {
    current_player_count: u64,
    max_players: u64,
    current_players: Vec<String>,
}

async fn get_current_online_players(server: &Server) -> Result<RconPlayersData> {
    trace!("Getting online players");
    let mut connection = rcon::Connection::connect(
        (server.server_ip.as_str(), server.rcon_port),
        &server.rcon_password,
    )
    .await?;
    trace!("rcon connected");

    let response = connection.cmd("list").await?;
    println!("{}", response);

    trace!(%response, "Got rcon response");

    trace!("built regex");

    let capts = LIST_RESPONSE_REGEX
        .captures(response.as_str())
        .ok_or_else(|| eyre::eyre!("Invalid response from `list`: {}", response))?;
    let current_player_count: u64 = capts[1].parse()?;
    let max_players: u64 = capts[2].parse()?;
    let current_players = capts
        .get(3)
        .into_iter()
        .map(|m| m.as_str().split_whitespace().map(|name| name.to_owned()))
        .flatten()
        .collect();
    Ok(RconPlayersData {
        current_player_count,
        max_players,
        current_players,
    })
}

pub async fn scrape_current_online_players<W: Write>(server: &Server, writer: &mut W) -> Result<()> {
    let results = get_current_online_players(server).await?;
    write_prometheus_blurb!(writer, "minecraft_online_player_count", "gauge");
    writeln!(
        writer,
        r#"minecraft_online_player_count{{server="{}"}} {}"#,
        server.server_name, results.current_player_count
    )?;

    write_prometheus_blurb!(writer, "minecraft_max_players", "gauge");
    writeln!(
        writer,
        r#"minecraft_max_players{{server="{}"}} {}"#,
        server.server_name, results.max_players
    )?;

    // TODO: Maybe use the collected list of players for something. The problem is that prometheus
    // doesn't support text.
    Ok(())
}
