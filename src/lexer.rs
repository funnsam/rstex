use std::collections::HashMap;

pub type Range = core::ops::Range<usize>;
pub type CowStr<'a> = std::borrow::Cow<'a, str>;

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub typ: TokenType,
    /// The range of the token's origin
    /// It is `0..0` if it does not come from a file
    pub range: Range,
    pub source: CowStr<'a>,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TokenType {
    /// This is a command if it happends to exist in the token stream
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
    pub catcodes: HashMap<char, TokenType>,
    chars: Option<String>,
}

impl<'a> Lexer<'a> {
    pub fn new(stream: &'a str) -> Self {
        Self {
            stream,
            range: 0..0,
            catcodes: HashMap::new(),
            chars: None,
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

    fn range(&self) -> Range {
        self.range.clone()
    }

    fn peek_type(&mut self) -> (Option<(TokenType, CowStr<'a>)>, usize) {
        let end = self.range.end;
        let typ = self.next_type();
        let diff = self.range.end - end;
        self.range.end = end;
        (typ, diff)
    }

    // NOTE: it only changes the end of range
    fn next_type(&mut self) -> Option<(TokenType, CowStr<'a>)> {
        let chr = self.next_char()?;
        self._next_type(chr, self.range.end - 1)
    }

    fn _next_type(&mut self, chr: char, start: usize) -> Option<(TokenType, CowStr<'a>)> {
        let typ = match chr {
            // c if let Some(t) = self.catcodes.get(&c) => t.clone(),
            c if self.catcodes.contains_key(&c) => self.catcodes[&c].clone(),

            '\\' => TokenType::Escape,
            '{' => TokenType::BeginGroup,
            '}' => TokenType::EndGroup,
            '$' => TokenType::MathShift,
            '&' => TokenType::AlignTab,
            '\n' => TokenType::Eol,
            '#' => TokenType::Parameter,
            '^' => {
                if matches!(self.peek_char(), Some('^')) {
                    self.range.end += '^'.len_utf8();
                    let next = self.next_char()?;
                    let chr = match next {
                        '0'..='9' | 'a'..='f' => {
                            let d = self.peek_char();
                            if matches!(d, Some('0'..='9' | 'a'..='f')) {
                                self.range.end += 1;

                                let digits = [next as u8, d.unwrap() as u8];
                                let digits = unsafe{ core::str::from_utf8_unchecked(&digits) };
                                digits.parse::<u8>().unwrap() as char
                            } else {
                                (next as u32 - b'@' as u32).try_into().unwrap()
                            }
                        },
                        _ => (next as u32 - b'@' as u32).try_into().unwrap(),
                    };

                    if let Some(s) = self.chars.as_mut() {
                        s.push(chr);
                    } else {
                        self.chars = Some(format!("{}{chr}", &self.stream[self.range.start..start].to_string()));
                    }

                    return self._next_type(chr, self.range.end).map(|(t, _)| (t, self.chars.as_ref().cloned().unwrap().into()));
                }

                TokenType::Superscript
            },
            '_' => TokenType::Subscript,
            '\0' | '\r' => TokenType::Ignored,
            ' ' | '\t' => {
                while let Some(c) = self.peek_char() {
                    if matches!(c, ' ' | '\t') || matches!(self.catcodes.get(&c), Some(TokenType::Space)) {
                        self.range.end += c.len_utf8();
                    } else {
                        break;
                    }
                }

                TokenType::Space
            },
            c if c.is_ascii_alphabetic() || c == '\x01' => TokenType::Letter,
            '~' => TokenType::Active,
            '%' => TokenType::Comment,
            '\x7f' => TokenType::Invalid,
            _ => TokenType::Other,
        };

        if let Some(s) = self.chars.as_mut() {
            *s += &self.stream[start..self.range.end];
            Some((typ, s.clone().into()))
        } else {
            Some((typ, self.stream[self.range()].into()))
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.start = self.range.end;
        self.chars = None;

        self.next_type().and_then(|t| match t.0 {
            TokenType::Comment => {
                while !matches!(self.next_type(), Some((TokenType::Eol, _)) | None) {}
                self.next()
            },
            TokenType::Escape => {
                let mut end = self.range.end;
                let next = self.next_type()?;
                let mut source = next.1;

                if matches!(next.0, TokenType::Letter) {
                    while let (Some((typ, src)), add) = self.peek_type() {
                        match typ {
                            TokenType::Letter => {
                                self.range.end += add;
                                end = self.range.end;
                                source = src;
                            },
                            TokenType::Space | TokenType::Eol => { self.range.end += add; break; }
                            _ => break,
                        }
                    }
                } else {
                    end = self.range.end;
                }

                Some(Token {
                    typ: TokenType::Escape,
                    range: self.range.start..end,
                    source,
                })
            },
            TokenType::Ignored => self.next(),
            TokenType::Invalid => todo!("reached invalid char"),
            typ => Some(Token {
                typ,
                range: self.range(),
                source: t.1,
            }),
        })
    }
}
