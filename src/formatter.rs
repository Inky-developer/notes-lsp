use phf::phf_map;
use tower_lsp_server::ls_types::{CompletionItem, CompletionItemKind, TextEdit};

use crate::syntax::{SyntaxKind, parse};

pub fn format(input: &str) -> Vec<TextEdit> {
    let syntax = parse(input);
    syntax
        .iter()
        .filter_map(|(range, node)| {
            let replacement = match node.kind {
                SyntaxKind::Text => None,
                SyntaxKind::Value { ident } => VALUE_REPLACEMENTS.get(ident).copied(),
                SyntaxKind::Super { ident } => SUPER_REPLACEMENTS.get(&ident).copied(),
                SyntaxKind::Sub { ident } => SUB_REPLACEMENTS.get(&ident).copied(),
            }?;
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
        SyntaxKind::Super { ident } => {
            if SUPER_REPLACEMENTS.contains_key(ident) {
                Vec::new()
            } else {
                SUPER_REPLACEMENTS
                    .entries()
                    .map(|(k, v)| CompletionItem {
                        label: k.to_string(),
                        kind: Some(CompletionItemKind::TEXT),
                        detail: Some(v.to_string()),
                        ..Default::default()
                    })
                    .collect()
            }
        }
        SyntaxKind::Sub { ident } => {
            if SUB_REPLACEMENTS.contains_key(ident) {
                Vec::new()
            } else {
                SUB_REPLACEMENTS
                    .entries()
                    .map(|(k, v)| CompletionItem {
                        label: k.to_string(),
                        kind: Some(CompletionItemKind::TEXT),
                        detail: Some(v.to_string()),
                        ..Default::default()
                    })
                    .collect()
            }
        }
        SyntaxKind::Text => Vec::new(),
    }
}

// The keys must all be alphanumeric due to the implementationof format
static VALUE_REPLACEMENTS: phf::Map<&'static str, &'static str> = phf_map! {
    // Lowercase Greek letters
    "alpha" => "α",
    "beta" => "β",
    "gamma" => "γ",
    "delta" => "δ",
    "epsilon" => "ε",
    "zeta" => "ζ",
    "eta" => "η",
    "theta" => "θ",
    "iota" => "ι",
    "kappa" => "κ",
    "lambda" => "λ",
    "mu" => "μ",
    "nu" => "ν",
    "xi" => "ξ",
    "omicron" => "ο",
    "pi" => "π",
    "rho" => "ρ",
    "sigma" => "σ",
    "tau" => "τ",
    "upsilon" => "υ",
    "phi" => "φ",
    "chi" => "χ",
    "psi" => "ψ",
    "omega" => "ω",
    // Uppercase Greek letters
    "Alpha" => "Α",
    "Beta" => "Β",
    "Gamma" => "Γ",
    "Delta" => "Δ",
    "Epsilon" => "Ε",
    "Zeta" => "Ζ",
    "Eta" => "Η",
    "Theta" => "Θ",
    "Iota" => "Ι",
    "Kappa" => "Κ",
    "Lambda" => "Λ",
    "Mu" => "Μ",
    "Nu" => "Ν",
    "Xi" => "Ξ",
    "Omicron" => "Ο",
    "Pi" => "Π",
    "Rho" => "Ρ",
    "Sigma" => "Σ",
    "Tau" => "Τ",
    "Upsilon" => "Υ",
    "Phi" => "Φ",
    "Chi" => "Χ",
    "Psi" => "Ψ",
    "Omega" => "Ω",
    // Logic symbols
    "forall" => "∀",
    "exists" => "∃",
    "nexists" => "∄",
    "in" => "∈",
    "notin" => "∉",
    "ni" => "∋",
    "and" => "∧",
    "or" => "∨",
    "not" => "¬",
    "implies" => "⇒",
    "iff" => "⇔",
    "top" => "⊤",
    "bot" => "⊥",
    "vdash" => "⊢",
    "models" => "⊨",
    "therefore" => "∴",
    "because" => "∵",
    // Set theory
    "intersect" => "∩",
    "union" => "∪",
    "subset" => "⊂",
    "subseteq" => "⊆",
    "supset" => "⊃",
    "supseteq" => "⊇",
    "emptyset" => "∅",
    "setminus" => "∖",
    // Calculus and analysis
    "infty" => "∞",
    "partial" => "∂",
    "nabla" => "∇",
    "sum" => "∑",
    "prod" => "∏",
    "int" => "∫",
    "sqrt" => "√",
    // Number sets (double-struck)
    "N" => "ℕ",
    "Z" => "ℤ",
    "Q" => "ℚ",
    "R" => "ℝ",
    "C" => "ℂ",
    "P" => "ℙ",
    "F" => "𝔽",
    // Some additional uncategorized symbols
    "blank" => "␣",
    "start" => "►",
};

static SUPER_REPLACEMENTS: phf::Map<char, &'static str> = phf_map! {
    '0' => "⁰",
    '1' => "¹",
    '2' => "²",
    '3' => "³",
    '4' => "⁴",
    '5' => "⁵",
    '6' => "⁶",
    '7' => "⁷",
    '8' => "⁸",
    '9' => "⁹",
    'a' => "ᵃ",
};

static SUB_REPLACEMENTS: phf::Map<char, &'static str> = phf_map! {
    '0' => "₀",
    '1' => "₁",
    '2' => "₂",
    '3' => "₃",
    '4' => "₄",
    '5' => "₅",
    '6' => "₆",
    '7' => "₇",
    '8' => "₈",
    '9' => "₉",
    'a' => "ₐ",
};
