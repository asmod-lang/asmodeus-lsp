use anyhow::Result;
use tower_lsp::{LspService, Server};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod error;

use error::LspError;

use tower_lsp::{Client, LanguageServer};
use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result as LspResult;

#[derive(Debug)]
pub struct AsmodeusLanguageServer {
    client: Client,
}

impl AsmodeusLanguageServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for AsmodeusLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Asmodeus LSP Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }
}

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

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| AsmodeusLanguageServer::new(client));
    
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
