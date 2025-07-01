use tower_lsp::lsp_types::*;

pub fn create_server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
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
                legend: create_semantic_tokens_legend(),
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
    }
}

fn create_semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::KEYWORD,      // instructions
            SemanticTokenType::FUNCTION,     // labels
            SemanticTokenType::NUMBER,       // numbers
            SemanticTokenType::OPERATOR,     // operators (#, [, ])
            SemanticTokenType::COMMENT,      // comments
            SemanticTokenType::STRING,       // strings
        ],
        token_modifiers: vec![
            SemanticTokenModifier::DEPRECATED, // not used currently
        ],
    }
}
