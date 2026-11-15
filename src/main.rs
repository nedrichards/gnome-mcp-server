mod config;
mod gnome;
mod mcp;
mod resources;
mod tools;

#[cfg(test)]
mod test_support;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting GNOME MCP server (stdio)");

    // Register as a host application, given that we use some portals.
    if let Err(err) =
        ashpd::register_host_app("com.belmoussaoui.gnome-mcp-server".try_into().unwrap()).await
    {
        tracing::warn!("Failed to register host app: {}", err);
    }

    mcp::Server::run().await
}
