use std::collections::HashMap;

pub type Range = core::ops::Range<usize>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Token {
    Escape,
    BeginGroup,
    EndGroup,
    MathShift,
    Align,
    Eol,
    MacroParam,
    Superscript,
    Subscript,
    Ignore,
    Space,
    Letters,
    Other,
    Active,
    Comment,
    Invalid,
}

pub struct Lexer<'a> {
    pub stream: &'a str,
    range: Range,
    pub catcodes: HashMap<char, Token>,
    was_escape: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(stream: &'a str) -> Self {
        Self {
            stream,
            range: 0..0,
            catcodes: HashMap::new(),
            was_escape: false,
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.stream.split_at_checked(self.range.end)?.1.chars().nth(0)
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.stream.split_at_checked(self.range.end)?.1.chars().nth(0)?;
        self.range.end += c.len_utf8();
        Some(c)
    }

    pub fn range(&self) -> Range {
        self.range.clone()
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let was_escape = core::mem::take(&mut self.was_escape);
        self.range.start = self.range.end;

        Some(match self.next_char()? {
            c if let Some(t) = self.catcodes.get(&c) => t.clone(),

            '\\' => { self.was_escape = true; Token::Escape },
            '{' => Token::BeginGroup,
            '}' => Token::EndGroup,
            '$' => Token::MathShift,
            '&' => Token::Align,
            '\n' => Token::Eol,
            '#' => Token::MacroParam,
            '^' => Token::Superscript,
            '_' => Token::Subscript,
            '\0' => Token::Ignore,
            ' ' => Token::Space,
            '~' => Token::Active,
            '%' => Token::Comment,
            '\x7f' => Token::Invalid,
            c if was_escape && c.is_ascii_alphabetic() => {
                while let Some(c) = self.peek_char() {
                    if c.is_ascii_alphabetic() || matches!(self.catcodes.get(&c), Some(Token::Letters)) {
                        self.range.end += 1;
                    } else {
                        break;
                    }
                }

                Token::Letters
            },
            c if c.is_ascii_alphabetic() => Token::Letters,
            _ => Token::Other,
        })
    }
}
