use phf::phf_map;
use tower_lsp_server::ls_types::TextEdit;

use crate::syntax::{SyntaxKind, parse};

pub fn format(input: &str) -> Vec<TextEdit> {
    let syntax = parse(input);
    syntax
        .iter()
        .filter_map(|(range, node)| match node.kind {
            SyntaxKind::Text => None,
            SyntaxKind::Value { ident } => find_replacement(ident)
                .map(|replacement| TextEdit::new(range, replacement.to_string())),
        })
        .collect()
}

// The keys must all be alphanumeric due to the implementationof format
static REPLACEMENTS: phf::Map<&'static str, &'static str> = phf_map! {
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

pub fn search_replacements(search: &str) -> impl Iterator<Item = (&'static str, &'static str)> {
    let search = search.strip_prefix("\\").unwrap_or(search);
    REPLACEMENTS
        .entries()
        .map(|(k, v)| (*k, *v))
        .filter(move |(key, _)| key.starts_with(search))
}

fn find_replacement(input: &str) -> Option<&'static str> {
    REPLACEMENTS.get(input).copied()
}
