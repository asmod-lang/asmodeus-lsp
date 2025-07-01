use tower_lsp::lsp_types::*;
use crate::analysis::refactoring::{QuickFixProvider, RefactoringActionsProvider};

#[derive(Debug)]
pub struct CodeActionsProvider {
    quick_fix_provider: QuickFixProvider,
    refactoring_provider: RefactoringActionsProvider,
}

impl CodeActionsProvider {
    pub fn new() -> Self {
        Self {
            quick_fix_provider: QuickFixProvider::new(),
            refactoring_provider: RefactoringActionsProvider::new(),
        }
    }

    pub fn get_code_actions(&self, content: &str, range: Range, uri: &Url, context: &CodeActionContext) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();
        
        // quick fixes for diagnostics
        for diagnostic in &context.diagnostics {
            if let Some(action) = self.quick_fix_provider.create_quick_fix(diagnostic, content, uri) {
                actions.push(action);
            }
        }
        
        // refactoring actions 
        actions.extend(self.refactoring_provider.get_refactoring_actions(content, range, uri));
        
        actions
    }
}
