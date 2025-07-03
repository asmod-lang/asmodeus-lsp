use crate::analysis::{DocumentState, SemanticAnalyzer};
use dashmap::DashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;

#[derive(Debug)]
pub struct RequestHandlers {
    analyzer: Arc<SemanticAnalyzer>,
    documents: Arc<DashMap<Url, DocumentState>>,
}

impl RequestHandlers {
    pub fn new(
        analyzer: Arc<SemanticAnalyzer>,
        documents: Arc<DashMap<Url, DocumentState>>,
    ) -> Self {
        Self {
            analyzer,
            documents,
        }
    }

    // Completion
    pub async fn handle_completion(
        &self,
        params: CompletionParams,
    ) -> LspResult<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let document = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let completions = self.analyzer.get_completions(&document.content, position);
        Ok(Some(CompletionResponse::Array(completions)))
    }

    // Hover
    pub async fn handle_hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(document) = self.documents.get(uri) {
            let hover_info = self.analyzer.get_hover_info(&document.content, position);
            return Ok(hover_info);
        }

        Ok(None)
    }

    // Go to definition
    pub async fn handle_goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> LspResult<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(document) = self.documents.get(uri) {
            let definition = self
                .analyzer
                .get_definition(&document.content, position, uri);
            return Ok(definition);
        }

        Ok(None)
    }

    // Find references
    pub async fn handle_references(
        &self,
        params: ReferenceParams,
    ) -> LspResult<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(document) = self.documents.get(uri) {
            let references = self.analyzer.find_references(
                &document.content,
                position,
                uri,
                params.context.include_declaration,
            );
            return Ok(Some(references));
        }

        Ok(Some(Vec::new()))
    }

    // Document symbols
    pub async fn handle_document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> LspResult<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        if let Some(document) = self.documents.get(uri) {
            let symbols = self.analyzer.get_document_symbols(&document.content);
            return Ok(Some(DocumentSymbolResponse::Flat(symbols)));
        }

        Ok(None)
    }

    // Workspace symbols
    pub async fn handle_workspace_symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> LspResult<Option<Vec<SymbolInformation>>> {
        let query = &params.query;
        let mut all_symbols = Vec::new();

        for document_ref in self.documents.iter() {
            let (uri_key, document) = document_ref.pair();
            let uri = uri_key.clone();

            let mut symbols = self.analyzer.get_document_symbols(&document.content);

            self.analyzer
                .filter_workspace_symbols(&mut symbols, query, &uri);

            all_symbols.extend(symbols);
        }

        Ok(Some(all_symbols))
    }

    // Semantic tokens
    pub async fn handle_semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> LspResult<Option<SemanticTokensResult>> {
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

    // Code actions
    pub async fn handle_code_action(
        &self,
        params: CodeActionParams,
    ) -> LspResult<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let range = params.range;

        if let Some(document) = self.documents.get(uri) {
            let actions =
                self.analyzer
                    .get_code_actions(&document.content, range, uri, &params.context);
            return Ok(Some(actions));
        }

        Ok(None)
    }

    // Signature help
    pub async fn handle_signature_help(
        &self,
        params: SignatureHelpParams,
    ) -> LspResult<Option<SignatureHelp>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(document) = self.documents.get(uri) {
            let signature = self
                .analyzer
                .get_signature_help(&document.content, position);
            return Ok(signature);
        }

        Ok(None)
    }

    // Prepare rename
    pub async fn handle_prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> LspResult<Option<PrepareRenameResponse>> {
        let uri = &params.text_document.uri;
        let position = params.position;

        if let Some(document) = self.documents.get(uri) {
            let range = self.analyzer.get_rename_range(&document.content, position);
            return Ok(range.map(PrepareRenameResponse::Range));
        }

        Ok(None)
    }

    // Rename
    pub async fn handle_rename(&self, params: RenameParams) -> LspResult<Option<WorkspaceEdit>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = &params.new_name;

        if let Some(document) = self.documents.get(uri) {
            let edit = self
                .analyzer
                .rename_symbol(&document.content, position, new_name, uri);
            return Ok(edit);
        }

        Ok(None)
    }
}
