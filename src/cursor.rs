use std::str::Chars;

use tower_lsp_server::ls_types::Position;

pub struct Cursor<'a> {
    chars: Chars<'a>,
    pub pos: Position,
    pub offset: usize,
}

impl<'a> From<&'a str> for Cursor<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            chars: value.chars(),
            pos: Position::default(),
            offset: 0,
        }
    }
}

impl<'a> Iterator for Cursor<'a> {
    type Item = (Position, usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let char = self.chars.next()?;
        let pos = self.pos;
        let offset = self.offset;
        match char {
            '\n' => {
                self.pos.line += 1;
                self.pos.character = 0;
            }
            _ => {
                self.pos.character += char.len_utf16() as u32;
            }
        };
        self.offset += char.len_utf8();
        Some((pos, offset, char))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.chars.size_hint()
    }
}
