use anyhow::Result;
use tower_lsp::{LspService, Server};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod error;
mod analysis;
mod server;

use server::AsmodeusLanguageServer;

#[tokio::main]
async fn main() -> Result<()> {
    // logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "asmodeus_lsp=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Asmodeus Language Server");

    // LSP
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| AsmodeusLanguageServer::new(client));
    
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
