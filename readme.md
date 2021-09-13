# minecraft-prometheus-exporter

A scraper for [Prometheus](https://prometheus.io/) for minecraft (1.13+) servers.

## Usage

First, enable rcon on your minecraft server, by setting the following values in `server.properties`:

```
rcon.port=<port>
rcon.password=<password>
enable-rcon=true
```

Then, create a configuration file. An example configuration file is provided in `config.yml`. Then run the program with the path to that file as the first argument e.g. `./target/release/minecraft-server-exporter config.yml`.

This program runs an http server, which Prometheus then polls. An example Prometheus configuration is provided:

```yml
scrape_configs:
  - job_name: "minecraft"
    scrape_timeout: 30s
    static_configs:
      - targets: ["localhost::9001"]
```

---

Available under the Mozilla Public Licence, version 2.0
