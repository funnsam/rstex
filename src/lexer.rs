use std::collections::HashMap;

pub type Range = core::ops::Range<usize>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Token {
    Escape,
    BeginGroup,
    EndGroup,
    MathShift,
    AlignTab,
    Eol,
    Parameter,
    Superscript,
    Subscript,
    Ignored,
    Space,
    Letter,
    Other,
    Active,
    Comment,
    Invalid,
}

pub struct Lexer<'a> {
    pub stream: &'a str,
    range: Range,
    pub catcodes: HashMap<char, Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(stream: &'a str) -> Self {
        Self {
            stream,
            range: 0..0,
            catcodes: HashMap::new(),
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
        self.range.start = self.range.end;

        Some(match self.next_char()? {
            // c if let Some(t) = self.catcodes.get(&c) => t.clone(),
            c if self.catcodes.contains_key(&c) => self.catcodes[&c].clone(),

            '\\' => Token::Escape,
            '{' => Token::BeginGroup,
            '}' => Token::EndGroup,
            '$' => Token::MathShift,
            '&' => Token::AlignTab,
            '\n' => Token::Eol,
            '#' => Token::Parameter,
            '^' => Token::Superscript,
            '_' => Token::Subscript,
            '\0' | '\r' => Token::Ignored,
            ' ' | '\t' => Token::Space,
            c if c.is_ascii_alphabetic() => Token::Letter,
            '~' => Token::Active,
            '%' => Token::Comment,
            '\x7f' => Token::Invalid,
            _ => Token::Other,
        })
    }
}
