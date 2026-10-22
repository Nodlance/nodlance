use std::error::Error;

use nodlance_mcp::run_stdio_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    eprintln!("Nodlance AI-Native Model Context Protocol (MCP) Server v0.1.0 started on stdio");
    run_stdio_server().await?;
    Ok(())
}
