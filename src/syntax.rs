use std::borrow::Cow;

use phf::phf_map;
use tower_lsp_server::ls_types::{Position, Range};

use crate::cursor::Cursor;

#[derive(Debug, PartialEq, Eq)]
pub enum SyntaxKind<'a> {
    Text,
    Value { ident: &'a str },
    Super { ident: &'a str },
    Sub { ident: char },
}

impl<'a> SyntaxKind<'a> {
    pub fn apply(&self) -> Option<Cow<'a, str>> {
        match self {
            SyntaxKind::Text => None,
            SyntaxKind::Value { ident } => Some(Cow::Borrowed(*VALUE_REPLACEMENTS.get(ident)?)),
            SyntaxKind::Super { ident } => Some(Cow::Owned(
                ident
                    .chars()
                    .map(|char| SUPER_REPLACEMENTS.get(&char).copied())
                    .collect::<Option<_>>()?,
            )),
            SyntaxKind::Sub { ident } => Some(Cow::Borrowed(*SUB_REPLACEMENTS.get(ident)?)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SyntaxNode<'a> {
    pub text: &'a str,
    pub kind: SyntaxKind<'a>,
}

pub struct SyntaxIter<'a, 'b> {
    start: Position,
    nodes: std::slice::Iter<'b, SyntaxNode<'a>>,
}

impl<'a, 'b> Iterator for SyntaxIter<'a, 'b> {
    type Item = (Range, &'b SyntaxNode<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        pub fn add_position(position: Position, text: &str) -> Position {
            let relative_pos = Cursor::from(text).run_to_end();
            Position {
                line: position.line + relative_pos.line,
                character: if relative_pos.line == 0 {
                    position.character + relative_pos.character
                } else {
                    relative_pos.character
                },
            }
        }

        let node = self.nodes.next()?;
        let start = self.start;
        let end = add_position(start, node.text);
        self.start = end;
        Some((Range::new(start, end), node))
    }
}

#[derive(Debug)]
pub struct Syntax<'a>(Vec<SyntaxNode<'a>>);

impl<'a> Syntax<'a> {
    pub fn iter<'b>(&'b self) -> impl Iterator<Item = (Range, &'b SyntaxNode<'a>)> {
        let nodes = self.0.iter();
        SyntaxIter {
            start: Position::default(),
            nodes,
        }
    }
}

pub fn parse<'a>(input: &'a str) -> Syntax<'a> {
    Syntax(Parser::new(input).parse())
}

struct Parser<'a> {
    input: &'a str,
    cursor: Cursor<'a>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            cursor: Cursor::from(input),
        }
    }

    fn peek(&self) -> char {
        self.input[self.cursor.offset..]
            .chars()
            .next()
            .unwrap_or('\0')
    }

    fn consume(&mut self) -> char {
        let char = self.peek();
        self.cursor.next();
        char
    }

    fn parse(&mut self) -> Vec<SyntaxNode<'a>> {
        let mut result = Vec::new();
        while self.peek() != '\0' {
            let node = self.parse_node();
            result.push(node);
        }
        result
    }

    fn is_start_of_node(&self) -> bool {
        matches!(self.peek(), '\\' | '^' | '_')
    }

    fn parse_node(&mut self) -> SyntaxNode<'a> {
        let start = self.cursor.offset;
        // Must be kept in sync with is_start_of_node
        let kind = match self.peek() {
            '\\' => self.parse_escape(),
            '^' => self.parse_super(),
            '_' => self.parse_sub(),
            _ => self.parse_text(),
        };
        // end will be calculated later
        SyntaxNode {
            text: &self.input[start..self.cursor.offset],
            kind,
        }
    }

    fn parse_text(&mut self) -> SyntaxKind<'a> {
        while self.peek() != '\0' && !self.is_start_of_node() {
            self.consume();
        }
        SyntaxKind::Text
    }

    fn parse_escape(&mut self) -> SyntaxKind<'a> {
        self.consume();

        // Allow escaping of backslashes
        if self.is_start_of_node() {
            self.consume();
            return SyntaxKind::Text;
        }

        let ident_start = self.cursor.offset;
        while self.peek().is_ascii_alphanumeric() {
            self.consume();
        }

        let ident = &self.input[ident_start..self.cursor.offset];
        SyntaxKind::Value { ident }
    }

    fn parse_super(&mut self) -> SyntaxKind<'a> {
        self.consume();

        let ident_start = self.cursor.offset;
        if self.peek() == '(' {
            let mut level = 0u32;

            while SUPER_REPLACEMENTS.contains_key(&self.peek()) {
                match self.peek() {
                    '(' => level += 1,
                    ')' => level -= 1,
                    _ => {}
                }
                self.consume();
                if level == 0 {
                    let ident = &self.input[(ident_start + 1)..(self.cursor.offset - 1)];
                    return if ident.is_empty() {
                        SyntaxKind::Text
                    } else {
                        SyntaxKind::Super { ident }
                    };
                }
            }
            // Either fully convert the parenthesis or do nothing
            return SyntaxKind::Text;
        }

        self.consume();
        let ident = &self.input[ident_start..self.cursor.offset];
        SyntaxKind::Super { ident }
    }

    fn parse_sub(&mut self) -> SyntaxKind<'a> {
        self.consume();
        let char = self.consume();
        // Heuristic to not replace characters in underscore words, like `parse_sub`
        if !self.peek().is_ascii_alphanumeric() {
            SyntaxKind::Sub { ident: char }
        } else {
            SyntaxKind::Text
        }
    }
}

// The keys must all be alphanumeric due to the implementationof format
pub(super) static VALUE_REPLACEMENTS: phf::Map<&'static str, &'static str> = phf_map! {
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
    "notsubset" => "⊄",
    "subseteq" => "⊆",
    "notsubseteq" => "⊈",
    "supset" => "⊃",
    "notsupset" => "⊅",
    "supseteq" => "⊇",
    "notsupseteq" => "⊉",
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
    'f' => "ᶠ",
    'k' => "ᵏ",
    'K' => "ᴷ",
    'n' => "ⁿ",
    'o' => "ᵒ",
    'O' => "ᴼ",
    '+' => "⁺",
    '-' => "⁻",
    '=' => "⁼",
    '(' => "⁽",
    ')' => "⁾",
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
    'i' => "ᵢ",
    'k' => "ₖ",
    'l' => "ₗ",
    'm' => "ₘ",
    'n' => "ₙ",
    'p' => "ₚ",
    's' => "ₛ",
    't' => "ₜ",
    'x' => "ₓ",
};

#[cfg(test)]
mod tests {
    use crate::syntax::{SyntaxKind, SyntaxNode, parse};

    #[test]
    fn test_parses_text() {
        assert_eq!(
            parse("Hello World!").0,
            vec![SyntaxNode {
                text: "Hello World!",
                kind: SyntaxKind::Text
            }]
        )
    }

    #[test]
    fn test_parses_function() {
        assert_eq!(
            parse("Hello \\World!").0,
            vec![
                SyntaxNode {
                    text: "Hello ",
                    kind: SyntaxKind::Text
                },
                SyntaxNode {
                    text: "\\World",
                    kind: SyntaxKind::Value { ident: "World" }
                },
                SyntaxNode {
                    text: "!",
                    kind: SyntaxKind::Text
                }
            ]
        )
    }
}
