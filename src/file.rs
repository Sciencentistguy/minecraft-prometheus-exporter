use std::fmt::Write;

use tracing::*;
use eyre::Result;
use walkdir::WalkDir;

use crate::config::Server;

pub fn scrape_server_file_size(server: &Server, writer: &mut impl Write) -> Result<()> {
    let server_root = server
        .stats_root
        .parent()
        .and_then(|x| x.parent())
        .expect("server root is stats_root/../..");

    trace!(?server_root, "Scraping server file size");

    let bytes_used: u64 = WalkDir::new(server_root)
        .into_iter()
        .map(|r| r.and_then(|entry| entry.metadata()).map(|meta| meta.len()))
        .collect::<Result<Vec<u64>, _>>()?
        .into_iter()
        .sum();

    write_prometheus_blurb!(writer, "minecraft_directory_size", "gauge");
    writeln!(
        writer,
        r#"minecraft_directory_size{{server="{}"}} {}"#,
        server.server_name, bytes_used
    )?;

    Ok(())
}
