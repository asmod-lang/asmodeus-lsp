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
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(false),
                trigger_characters: Some(vec![" ".to_string(), "\t".to_string()]),
                work_done_progress_options: Default::default(),
                all_commit_characters: None,
                completion_item: None,
            }),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            semantic_tokens_provider: Some( 
                SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                    legend: SemanticTokensLegend {
                    token_types: vec![
                        SemanticTokenType::KEYWORD,      // instructions
                        SemanticTokenType::FUNCTION,     // labels
                        SemanticTokenType::NUMBER,       // numbers
                        SemanticTokenType::OPERATOR,     // operators (#, [, ])
                        SemanticTokenType::COMMENT,      // comments
                        SemanticTokenType::STRING,       // strings
                ],
                    token_modifiers: vec![
                        SemanticTokenModifier::DEPRECATED, // not used
                ],
            },
                range: Some(true),
                full: Some(SemanticTokensFullOptions::Bool(true)),
            ..Default::default()
        })
    ),
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            workspace_symbol_provider: Some(OneOf::Left(true)),
            rename_provider: Some(OneOf::Left(true)),
            signature_help_provider: Some(SignatureHelpOptions { 
                trigger_characters: Some(vec![" ".to_string()]),
                retrigger_characters: Some(vec![",".to_string()]),
                work_done_progress_options: Default::default(),
            }),   
            ..Default::default()
        },
            ..Default::default()
        })
    }

async fn signature_help(&self, params: SignatureHelpParams) -> tower_lsp::jsonrpc::Result<Option<SignatureHelp>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    if let Some(document) = self.documents.get(uri) {
        let signature = self.analyzer.get_signature_help(&document.content, position);
        return Ok(signature);
    }

    Ok(None)
}


async fn symbol(&self, params: WorkspaceSymbolParams) -> tower_lsp::jsonrpc::Result<Option<Vec<SymbolInformation>>> {
    let query = &params.query;
    let mut all_symbols = Vec::new();
    
    // all open documents
    for document_ref in self.documents.iter() {
        let (uri_key, document) = document_ref.pair();
        let uri = uri_key.clone();
        
        let mut symbols = self.analyzer.get_document_symbols(&document.content);
        
        // symbols by query and update URIs
        for symbol in &mut symbols {
            if symbol.name.to_lowercase().contains(&query.to_lowercase()) {
                symbol.location.uri = uri.clone();
                all_symbols.push(symbol.clone());
            }
        }
    }
    
    Ok(Some(all_symbols))
}

async fn prepare_rename(&self, params: TextDocumentPositionParams) -> tower_lsp::jsonrpc::Result<Option<PrepareRenameResponse>> {
    let uri = &params.text_document.uri;
    let position = params.position;
    
    if let Some(document) = self.documents.get(uri) {
        let range = self.analyzer.get_rename_range(&document.content, position);
        return Ok(range.map(PrepareRenameResponse::Range));
    }
    
    Ok(None)
}

async fn rename(&self, params: RenameParams) -> tower_lsp::jsonrpc::Result<Option<WorkspaceEdit>> {
    let uri = &params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;
    let new_name = &params.new_name;
    
    if let Some(document) = self.documents.get(uri) {
        let edit = self.analyzer.rename_symbol(&document.content, position, new_name, uri);
        return Ok(edit);
    }
    
    Ok(None)
}

async fn code_action(&self, params: CodeActionParams) -> tower_lsp::jsonrpc::Result<Option<CodeActionResponse>> {
    let uri = &params.text_document.uri;
    let range = params.range;

    if let Some(document) = self.documents.get(uri) {
        let actions = self.analyzer.get_code_actions(&document.content, range, uri, &params.context);
        return Ok(Some(actions));
    }

    Ok(None)
}

async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> tower_lsp::jsonrpc::Result<Option<SemanticTokensResult>> {
    let uri = &params.text_document.uri;

    if let Some(document) = self.documents.get(uri) {
        let tokens = self.analyzer.get_semantic_tokens(&document.content);
        return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        })));
    }

    Ok(None)
}

async fn references(&self, params: ReferenceParams) -> tower_lsp::jsonrpc::Result<Option<Vec<Location>>> {
    let uri = &params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;

    if let Some(document) = self.documents.get(uri) {
        let references = self.analyzer.find_references(&document.content, position, uri, params.context.include_declaration);
        return Ok(Some(references));
    }

    Ok(Some(Vec::new()))
}

async fn document_symbol(&self, params: DocumentSymbolParams) -> tower_lsp::jsonrpc::Result<Option<DocumentSymbolResponse>> {
    let uri = &params.text_document.uri;

    if let Some(document) = self.documents.get(uri) {
        let symbols = self.analyzer.get_document_symbols(&document.content);
        return Ok(Some(DocumentSymbolResponse::Flat(symbols)));
    }

    Ok(None)
}

async fn goto_definition(&self, params: GotoDefinitionParams) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    if let Some(document) = self.documents.get(uri) {
        let definition = self.analyzer.get_definition(&document.content, position, uri);
        return Ok(definition);
    }

    Ok(None)
}

async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    if let Some(document) = self.documents.get(uri) {
        let hover_info = self.analyzer.get_hover_info(&document.content, position);
        return Ok(hover_info);
    }

    Ok(None)
}

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Asmodeus LSP Server initialized!")
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // document content
        let document = match self.documents.get(uri) {
            Some(doc) => doc,
            None => {
                return Ok(None);
            }
        };

        let completions = self.analyzer.get_completions(&document.content, position);

        Ok(Some(CompletionResponse::Array(completions)))
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
