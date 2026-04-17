use tower_lsp_server::ls_types::{Position, Range};

use crate::cursor::Cursor;

#[derive(Debug, PartialEq, Eq)]
pub enum SyntaxKind<'a> {
    Text,
    Value { ident: &'a str },
}

#[derive(Debug, PartialEq, Eq)]
pub struct SyntaxNode<'a> {
    pub text: &'a str,
    pub end_exclusive: Position,
    pub kind: SyntaxKind<'a>,
}

pub struct SyntaxIter<'a, 'b> {
    start: Position,
    nodes: std::slice::Iter<'b, SyntaxNode<'a>>,
}

impl<'a, 'b> Iterator for SyntaxIter<'a, 'b> {
    type Item = (Range, &'b SyntaxNode<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.nodes.next()?;
        let start = self.start;
        self.start = node.end_exclusive;
        Some((Range::new(start, node.end_exclusive), node))
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

    fn peek(&self) -> (char, Position) {
        let char = self.input[self.cursor.offset..]
            .chars()
            .next()
            .unwrap_or('\0');
        (char, self.cursor.pos)
    }

    fn consume(&mut self) -> (char, Position) {
        let char = self.peek();
        self.cursor.next();
        char
    }

    fn parse(&mut self) -> Vec<SyntaxNode<'a>> {
        let mut result = Vec::new();
        while self.peek().0 != '\0' {
            let node = self.parse_node();
            result.push(node);
        }
        result
    }

    fn is_start_of_node(&self) -> bool {
        matches!(self.peek().0, '\\')
    }

    fn parse_node(&mut self) -> SyntaxNode<'a> {
        let start = self.cursor.offset;
        // Must be kept in sync with is_start_of_node
        let kind = match self.peek().0 {
            '\\' => self.parse_escape(),
            _ => self.parse_text(),
        };
        // end will be calculated later
        SyntaxNode {
            text: &self.input[start..self.cursor.offset],
            end_exclusive: self.peek().1,
            kind,
        }
    }

    fn parse_text(&mut self) -> SyntaxKind<'a> {
        while self.peek().0 != '\0' && !self.is_start_of_node() {
            self.consume();
        }
        SyntaxKind::Text
    }

    fn parse_escape(&mut self) -> SyntaxKind<'a> {
        self.consume();

        // Allow escaping of backslashes
        if self.peek().0 == '\\' {
            self.consume();
            return SyntaxKind::Text;
        }

        let ident_start = self.cursor.offset;
        while self.peek().0.is_ascii_alphanumeric() {
            self.consume();
        }

        let ident = &self.input[ident_start..self.cursor.offset];
        SyntaxKind::Value { ident }
    }
}

#[cfg(test)]
mod tests {
    use tower_lsp_server::ls_types::Position;

    use crate::syntax::{SyntaxKind, SyntaxNode, parse};

    #[test]
    fn test_parses_text() {
        assert_eq!(
            parse("Hello World!").0,
            vec![SyntaxNode {
                text: "Hello World!",
                end_exclusive: Position::new(0, 12),
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
                    end_exclusive: Position::new(0, 6),
                    kind: SyntaxKind::Text
                },
                SyntaxNode {
                    text: "\\World",
                    end_exclusive: Position::new(0, 12),
                    kind: SyntaxKind::Value { ident: "World" }
                },
                SyntaxNode {
                    text: "!",
                    end_exclusive: Position::new(0, 13),
                    kind: SyntaxKind::Text
                }
            ]
        )
    }
}
