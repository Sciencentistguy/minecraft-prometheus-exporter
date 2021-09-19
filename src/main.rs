mod config;
mod player;
#[macro_use]
mod macros;
mod file;
mod rcon;

use std::path::PathBuf;

use eyre::Result;
use once_cell::sync::Lazy;
use structopt::StructOpt;
use tracing::Level;
use tracing::*;
use warp::Filter;

use crate::config::Config;

static OPTIONS: Lazy<Opt> = Lazy::new(Opt::from_args);
static CONFIG: Lazy<Config> = Lazy::new(|| {
    Config::open_or_create().unwrap_or_else(|e| {
        error!(error = ?e, "An error ocurred while opening config file");
        panic!("{:?}", e);
    })
});

#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .pretty()
            .finish(),
    )?;

    better_panic::install();

    Lazy::force(&CONFIG);

    let filter = warp::path("metrics").and_then(|| async move {
        info!("Received an incoming connection to `/metrics`");
        if false {
            // XXX: Type inference hint, I kinda hate it.
            return Err(warp::reject());
        }
        let mut output = String::new();
        for server in &CONFIG.servers {
            file::scrape_server_file_size(server, &mut output).map_err(|e| {
                error!(error = ?e, "An error ocurred in scrape_server_file_size");
                warp::reject::reject()
            })?;
            rcon::scrape_current_online_players(server, &mut output)
                .await
                .map_err(|e| {
                    error!(error = ?e, "An error ocurred in scrape_online_players");
                    warp::reject::reject()
                })?;
            player::scrape_player_stats(server, &mut output)
                .await
                .map_err(|e| {
                    error!(error = ?e, "An error ocurred in scrape_server");
                    warp::reject::reject()
                })?;
        }

        Ok(output)
    });

    info!(port = %CONFIG.port, "Started listening.");
    warp::serve(filter).run(([127, 0, 0, 1], CONFIG.port)).await;
    Ok(())
}

#[derive(Debug, StructOpt)]
struct Opt {
    /// The configuration file, in YAML
    config_file: PathBuf,
}
