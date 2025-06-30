use anyhow::Result;
use tower_lsp::{LspService, Server};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;
use dashmap::DashMap;

mod error;
mod analysis;

use analysis::{SemanticAnalyzer, DocumentState};

use tower_lsp::{Client, LanguageServer};
use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result as LspResult;

#[derive(Debug)]
pub struct AsmodeusLanguageServer {
    client: Client,
    analyzer: Arc<SemanticAnalyzer>,
    documents: Arc<DashMap<Url, DocumentState>>,
}

impl AsmodeusLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            analyzer: Arc::new(SemanticAnalyzer::new()),
            documents: Arc::new(DashMap::new()),
        }
    }

    async fn analyze_and_publish_diagnostics(&self, uri: &Url, content: &str) {
        let diagnostics = self.analyzer.analyze_document(content, uri);
        
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
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

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        // store document
        let document = DocumentState::new(uri.clone(), content.clone(), version);
        self.documents.insert(uri.clone(), document);

        self.analyze_and_publish_diagnostics(&uri, &content).await;

        self.client
            .log_message(MessageType::INFO, format!("Opened document: {}", uri))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.into_iter().next() {
            let content = change.text;

            // update document
            if let Some(mut document) = self.documents.get_mut(&uri) {
                document.update_content(content.clone(), version);
            }

            self.analyze_and_publish_diagnostics(&uri, &content).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, format!("Saved document: {}", params.text_document.uri))
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
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
