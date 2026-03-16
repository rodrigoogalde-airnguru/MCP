mod server;
mod tools;
mod models;
mod utils;

use anyhow::Result;
use server::Weather;

#[tokio::main]
async fn main() -> Result<()> {
    // Crear el servidor MCP
    let server = Weather::new();

    // Iniciar el servidor con stdin/stdout
    // El servidor escucha en stdin y responde en stdout
    server.run().await?;

    Ok(())
}
