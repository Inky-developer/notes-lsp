use phf::phf_map;
use tower_lsp_server::ls_types::{Position, Range, TextEdit};

use crate::cursor::Cursor;

pub fn format(mut input: String) -> Vec<TextEdit> {
    fn find_candidates(cursor: Cursor, mut on_candidate: impl FnMut(&str, Range)) {
        let input = cursor.as_str();
        let mut start_pos: Option<(usize, Position)> = None;
        for (position, offset, char) in cursor {
            match (start_pos, char) {
                (None, '\\') => {
                    start_pos = Some((offset, position));
                }
                (Some((start, start_position)), char) if !char.is_ascii_alphanumeric() => {
                    if char == '\\'
                        && start_position.line == position.line
                        && start_position.character + 1 == position.character
                    {
                        // escape
                        start_pos = None;
                        continue;
                    }
                    let range = Range::new(start_position, position);
                    let view = &input[start..offset];
                    on_candidate(view, range);
                    start_pos = if char == '\\' {
                        Some((offset, position))
                    } else {
                        None
                    };
                }
                _ => {}
            }
        }
    }

    let mut results = Vec::new();
    // Add zero at the end so that the last candidate will be found
    input.push(char::MIN);
    find_candidates(Cursor::from(input.as_str()), |candidate, range| {
        if let Some(replacement) = find_replacement(candidate) {
            results.push(TextEdit::new(range, replacement.to_string()));
        }
    });
    results
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
};

pub fn search_replacements(search: &str) -> impl Iterator<Item = (&'static str, &'static str)> {
    let search = search.strip_prefix("\\").unwrap_or(search);
    REPLACEMENTS
        .entries()
        .map(|(k, v)| (*k, *v))
        .filter(move |(key, _)| key.starts_with(search))
}

fn find_replacement(input: &str) -> Option<&'static str> {
    let input = input.strip_prefix("\\").expect("Should have that prefix!");
    REPLACEMENTS.get(input).copied()
}
