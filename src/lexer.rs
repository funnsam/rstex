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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    fn catcode_of(&self, chr: char) -> TokenType {
        match chr {
            // c if let Some(t) = self.catcodes.get(&c) => t.clone(),
            c if self.catcodes.contains_key(&c) => self.catcodes[&c].clone(),

            '\\' => TokenType::Escape,
            '{' => TokenType::BeginGroup,
            '}' => TokenType::EndGroup,
            '$' => TokenType::MathShift,
            '&' => TokenType::AlignTab,
            '\n' => TokenType::Eol,
            '#' => TokenType::Parameter,
            '^' => TokenType::Superscript,
            '_' => TokenType::Subscript,
            '\0' | '\r' => TokenType::Ignored,
            ' ' | '\t' => TokenType::Space,
            c if c.is_ascii_alphabetic() => TokenType::Letter,
            '~' => TokenType::Active,
            '%' => TokenType::Comment,
            '\x7f' => TokenType::Invalid,
            _ => TokenType::Other,
        }
    }

    fn peek_type(&mut self, s_state: bool) -> (Option<(TokenType, CowStr<'a>)>, usize) {
        let end = self.range.end;
        let typ = self.next_type(s_state);
        let diff = self.range.end - end;
        self.range.end = end;
        (typ, diff)
    }

    // NOTE: it only changes the end of range
    fn next_type(&mut self, s_state: bool) -> Option<(TokenType, CowStr<'a>)> {
        let chr = self.next_char()?;
        self._next_type(chr, s_state, self.range.end - 1)
    }

    fn _next_type(&mut self, chr: char, s_state: bool, start: usize) -> Option<(TokenType, CowStr<'a>)> {
        let typ = self.catcode_of(chr);

        match typ {
            TokenType::Ignored => return self.next_type(s_state),
            TokenType::Space | TokenType::Eol if s_state => return self.next_type(s_state),
            TokenType::Space => {
                while let Some(c) = self.peek_char() {
                    match self.catcode_of(c) {
                        TokenType::Space => {
                            self.range.end += c.len_utf8();
                        },
                        TokenType::Eol => {
                            self.range.end += c.len_utf8();
                            return self.next_type(s_state);
                        },
                        _ => break,
                    }
                }
            },
            TokenType::Superscript => {
                if self.peek_char() == Some(chr) {
                    self.range.end += chr.len_utf8();
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
                                (next as u8).wrapping_sub(b'@') as _
                            }
                        },
                        _ => (next as u8).wrapping_sub(b'@') as _
                    };

                    if let Some(s) = self.chars.as_mut() {
                        s.push(chr);
                    } else {
                        self.chars = Some(format!("{}{chr}", &self.stream[self.range.start..start].to_string()));
                    }

                    return self._next_type(chr, false, self.range.end).map(|(t, _)| (t, self.chars.as_ref().cloned().unwrap().into()));
                }
            },
            _ => {}
        }

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

        self.next_type(false).and_then(|t| match t.0 {
            TokenType::Comment => {
                while !matches!(self.next_type(false), Some((TokenType::Eol, _)) | None) {}
                self.next()
            },
            TokenType::Escape => {
                let mut end = self.range.end;
                let next = self.next_type(false)?;
                let mut source = next.1;

                if matches!(next.0, TokenType::Letter) {
                    loop {
                        if let Some(c) = self.peek_char() {
                            if matches!(self.catcode_of(c), TokenType::Space | TokenType::Eol) {
                                self.next_type(true);
                                break;
                            }
                        } else {
                            break;
                        }

                        if let (Some((typ, src)), add) = self.peek_type(true) {
                            match typ {
                                TokenType::Letter => {
                                    self.range.end += add;
                                    end = self.range.end;
                                    source = src;
                                },
                                _ => break,
                            }
                        } else {
                            break;
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
