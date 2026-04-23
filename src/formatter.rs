use tower_lsp_server::ls_types::{CompletionItem, CompletionItemKind, TextEdit};

use crate::syntax::{SyntaxKind, VALUE_REPLACEMENTS, parse};

pub fn format(input: &str) -> Vec<TextEdit> {
    let syntax = parse(input);
    syntax
        .iter()
        .filter_map(|(range, node)| {
            let replacement = node.kind.apply()?;
            Some(TextEdit::new(range, replacement.to_string()))
        })
        .collect()
}

pub fn get_completions(kind: &SyntaxKind) -> Vec<CompletionItem> {
    match kind {
        SyntaxKind::Value { ident } => {
            let suggestions = VALUE_REPLACEMENTS
                .entries()
                .map(|(k, v)| (*k, *v))
                .filter(move |(key, _)| key.starts_with(ident));
            let completions = suggestions.map(|(k, v)| CompletionItem {
                label: k.to_string(),
                kind: Some(CompletionItemKind::TEXT),
                detail: Some(v.to_string()),
                ..Default::default()
            });
            completions.collect()
        }
        SyntaxKind::Text | SyntaxKind::Super { .. } | SyntaxKind::Sub { .. } => Vec::new(),
    }
}
