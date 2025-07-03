use crate::analysis::{
    core::DiagnosticsEngine,
    language::{
        CompletionProvider, HoverProvider, NavigationProvider, SemanticTokensProvider,
        SignatureHelpProvider, SymbolProvider, ValidationProvider,
    },
    refactoring::{CodeActionsProvider, RenameProvider},
};
use tower_lsp::lsp_types::*;

#[derive(Debug)]
pub struct SemanticAnalyzer {
    diagnostics_engine: DiagnosticsEngine,
    completion_provider: CompletionProvider,
    hover_provider: HoverProvider,
    navigation_provider: NavigationProvider,
    symbol_provider: SymbolProvider,
    semantic_tokens_provider: SemanticTokensProvider,
    signature_help_provider: SignatureHelpProvider,
    code_actions_provider: CodeActionsProvider,
    rename_provider: RenameProvider,
    validation_provider: ValidationProvider,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            diagnostics_engine: DiagnosticsEngine::new(),
            completion_provider: CompletionProvider::new(),
            hover_provider: HoverProvider::new(),
            navigation_provider: NavigationProvider::new(),
            symbol_provider: SymbolProvider::new(),
            semantic_tokens_provider: SemanticTokensProvider::new(),
            signature_help_provider: SignatureHelpProvider::new(),
            code_actions_provider: CodeActionsProvider::new(),
            rename_provider: RenameProvider::new(),
            validation_provider: ValidationProvider::new(),
        }
    }

    // Diagnostics
    pub fn analyze_document(&self, content: &str, uri: &Url) -> Vec<Diagnostic> {
        let mut diagnostics = self.diagnostics_engine.analyze_document(content, uri);

        diagnostics.extend(self.validation_provider.validate_symbol_usage(content));

        diagnostics
    }

    // Completion
    pub fn get_completions(&self, content: &str, position: Position) -> Vec<CompletionItem> {
        self.completion_provider.get_completions(content, position)
    }

    // Hover
    pub fn get_hover_info(&self, content: &str, position: Position) -> Option<Hover> {
        self.hover_provider.get_hover_info(content, position)
    }

    // Navigation
    pub fn get_definition(
        &self,
        content: &str,
        position: Position,
        uri: &Url,
    ) -> Option<GotoDefinitionResponse> {
        self.navigation_provider
            .get_definition(content, position, uri)
    }

    pub fn find_references(
        &self,
        content: &str,
        position: Position,
        uri: &Url,
        include_declaration: bool,
    ) -> Vec<Location> {
        self.navigation_provider
            .find_references(content, position, uri, include_declaration)
    }

    // Symbols
    pub fn get_document_symbols(&self, content: &str) -> Vec<SymbolInformation> {
        self.symbol_provider.get_document_symbols(content)
    }

    pub fn get_rename_range(&self, content: &str, position: Position) -> Option<Range> {
        self.rename_provider.get_rename_range(content, position)
    }

    // Semantic tokens
    pub fn get_semantic_tokens(&self, content: &str) -> Vec<SemanticToken> {
        self.semantic_tokens_provider.get_semantic_tokens(content)
    }

    // Signature help
    pub fn get_signature_help(&self, content: &str, position: Position) -> Option<SignatureHelp> {
        self.signature_help_provider
            .get_signature_help(content, position)
    }

    // Code actions
    pub fn get_code_actions(
        &self,
        content: &str,
        range: Range,
        uri: &Url,
        context: &CodeActionContext,
    ) -> Vec<CodeActionOrCommand> {
        self.code_actions_provider
            .get_code_actions(content, range, uri, context)
    }

    // Rename
    pub fn rename_symbol(
        &self,
        content: &str,
        position: Position,
        new_name: &str,
        uri: &Url,
    ) -> Option<WorkspaceEdit> {
        self.rename_provider
            .rename_symbol(content, position, new_name, uri)
    }

    // Filter workspace Symbols
    pub fn filter_workspace_symbols(
        &self,
        symbols: &mut Vec<SymbolInformation>,
        query: &str,
        uri: &Url,
    ) {
        self.symbol_provider
            .filter_workspace_symbols(symbols, query, uri)
    }
}
