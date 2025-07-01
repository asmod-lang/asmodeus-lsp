use tower_lsp::{Client, LanguageServer};
use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result as LspResult;
use std::sync::Arc;
use dashmap::DashMap;

use crate::analysis::{SemanticAnalyzer, DocumentState};
use crate::server::{capabilities::create_server_capabilities, handlers::RequestHandlers};

#[derive(Debug)]
pub struct AsmodeusLanguageServer {
    client: Client,
    analyzer: Arc<SemanticAnalyzer>,
    documents: Arc<DashMap<Url, DocumentState>>,
    handlers: RequestHandlers,
}

impl AsmodeusLanguageServer {
    pub fn new(client: Client) -> Self {
        let analyzer = Arc::new(SemanticAnalyzer::new());
        let documents = Arc::new(DashMap::new());
        let handlers = RequestHandlers::new(analyzer.clone(), documents.clone());

        Self {
            client,
            analyzer,
            documents,
            handlers,
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
            capabilities: create_server_capabilities(),
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

    // document lifecycle
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        // save document
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

    // Language features - handlers delegation
    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        self.handlers.handle_completion(params).await
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        self.handlers.handle_hover(params).await
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> LspResult<Option<GotoDefinitionResponse>> {
        self.handlers.handle_goto_definition(params).await
    }

    async fn references(&self, params: ReferenceParams) -> LspResult<Option<Vec<Location>>> {
        self.handlers.handle_references(params).await
    }

    async fn document_symbol(&self, params: DocumentSymbolParams) -> LspResult<Option<DocumentSymbolResponse>> {
        self.handlers.handle_document_symbol(params).await
    }

    async fn symbol(&self, params: WorkspaceSymbolParams) -> LspResult<Option<Vec<SymbolInformation>>> {
        self.handlers.handle_workspace_symbol(params).await
    }

    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> LspResult<Option<SemanticTokensResult>> {
        self.handlers.handle_semantic_tokens_full(params).await
    }

    async fn code_action(&self, params: CodeActionParams) -> LspResult<Option<CodeActionResponse>> {
        self.handlers.handle_code_action(params).await
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> LspResult<Option<SignatureHelp>> {
        self.handlers.handle_signature_help(params).await
    }

    async fn prepare_rename(&self, params: TextDocumentPositionParams) -> LspResult<Option<PrepareRenameResponse>> {
        self.handlers.handle_prepare_rename(params).await
    }

    async fn rename(&self, params: RenameParams) -> LspResult<Option<WorkspaceEdit>> {
        self.handlers.handle_rename(params).await
    }
}
