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

#[derive(Clone)]
pub struct Lexer<'a, L: 'a + Clone + Iterator<Item = &'a str>> {
    pub stream: L,
    cur_line: Option<(usize, &'a str)>,
    pub catcodes: HashMap<char, TokenType>,
    range: Range,
    state: State,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    #[default] N, M, S
}

impl<'a, L: 'a + Clone + Iterator<Item = &'a str>> Lexer<'a, L> {
    pub fn new(mut stream: L) -> Self {
        let line = stream.next();

        Self {
            stream,
            cur_line: line.map(|l| (0, l.trim_end())),
            catcodes: HashMap::new(),
            range: 0..0,
            state: State::N
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        if self.range.end - self.cur_line?.0 > self.cur_line?.1.len() {
            // println!("reload {:?} {:?}", self.range, self.cur_line);
            self.cur_line = self.stream.next().map(|l| (self.range.end, l.trim_end()));
            // println!("{:?}", self.cur_line);
        }

        if self.range.end - self.cur_line?.0 == self.cur_line?.1.len() {
            // println!("nl {:?}", self.range);
            return Some('\n');
        }

        let c = self.cur_line?.1.split_at_checked(self.range.end - self.cur_line?.0)?.1.chars().nth(0);
        // println!("{c:?} {:?}", self.range);
        c
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek_char()?;
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

    // fn peek_next(&mut self) -> (Option<Token<'a>>, usize) {
    //     let mut ns = self.clone();
    //     println!("peek");
    //     let typ = ns.next();
    //     let diff = ns.range.end - self.range.end;
    //     println!("{typ:?} {diff}");
    //     (typ, diff)
    // }

    fn is_ss_char(&mut self, c: char) -> bool {
        self.peek_char() == Some(c)                                 // followed by identical character
            && self.next_char().is_some()                           // placeholder to skip 1 char
            && self.peek_char().map_or(false, |c| (c as u32) < 128) // c < 128
    }

    fn get_ss_char(&mut self) -> char {
        let next = self.next_char().unwrap();
        match next {
            '0'..='9' | 'a'..='f' if self.peek_char().map_or(false, |c| matches!(c, '0'..='9' | 'a'..='f')) => {
                let hex = [next as u8, self.next_char().unwrap() as u8];
                u8::from_str_radix(unsafe { core::str::from_utf8_unchecked(&hex) }, 16).unwrap() as char
            }
            _ => {
                (next as u8).wrapping_sub(64) as char
            },
        }
    }

    fn _next(&mut self, c: char) -> Option<Token<'a>> {
        let typ = self.catcode_of(c);
        match typ {
            TokenType::Escape => {
                match self.peek_char().map(|c| (c, self.catcode_of(c))) {
                    Some((_, TokenType::Eol)) | None => {
                        self.state = State::M;
                        Some(Token { typ: TokenType::Escape, range: self.range(), source: "".into() })
                    },
                    Some((_, TokenType::Letter)) => {
                        let mut source = String::new();

                        loop {
                            let next = self.peek_char();
                            match next.map(|c| (c, self.catcode_of(c))) {
                                Some((_, TokenType::Letter)) => {
                                    let tok = self.next().unwrap();
                                    debug_assert_eq!(tok.typ, TokenType::Letter);
                                    source += &tok.source;
                                },
                                Some((c, TokenType::Superscript)) => {
                                    self.range.end += c.len_utf8();
                                    let is_ss = self.is_ss_char(c);
                                    self.range.end -= c.len_utf8();

                                    if is_ss {
                                        if let Some(Token { typ: TokenType::Letter, source: src, .. }) = self.next() {
                                            source += &src;
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                                _ => break,
                            }
                        }

                        self.state = State::S;
                        Some(Token {
                            typ: TokenType::Escape,
                            range: self.range(),
                            source: source.into(),
                        })
                    },
                    Some((c, _)) => {
                        self.state = State::M;
                        self.range.end += c.len_utf8();
                        Some(Token {
                            typ: TokenType::Escape,
                            range: self.range(),
                            source: "".into(), //self.stream[self.range.end - c.len_utf8()..self.range.end].into(),
                        })
                    },
                }
            },
            TokenType::Superscript if self.is_ss_char(c) => {
                // let range = self.range();
                let c = self.get_ss_char();
                let mut next = self._next(c)?;
                // if next.source == &self.stream[range] {
                //     next.source = c.to_string().into();
                // }
                Some(next)
            },
            TokenType::BeginGroup |
            TokenType::EndGroup |
            TokenType::MathShift |
            TokenType::AlignTab |
            TokenType::Parameter |
            TokenType::Superscript |
            TokenType::Subscript |
            TokenType::Letter |
            TokenType::Other |
            TokenType::Active => {
                self.state = State::M;
                // Some(Token { typ, range: self.range(), source: self.stream[self.range()].into() })
                // TODO: allocless
                Some(Token { typ, range: self.range(), source: c.to_string().into() })
            },
            TokenType::Eol => {
                match core::mem::take(&mut self.state) {
                    State::N => Some(Token { typ: TokenType::Escape, range: self.range(), source: "par".into() }),
                    State::M => Some(Token { typ: TokenType::Space, range: self.range(), source: " ".into() }),
                    State::S => self.next(),
                }
            },
            TokenType::Ignored => self.next(),
            TokenType::Space => {
                match self.state {
                    State::N | State::S => self.next(),
                    State::M => {
                        self.state = State::S;
                        Some(Token { typ: TokenType::Space, range: self.range(), source: " ".into() })
                    },
                }
            },
            TokenType::Comment => {
                while self.next_char().map_or(false, |c| self.catcode_of(c) != TokenType::Eol) {}
                self.next()
            },
            TokenType::Invalid => {
                println!("! encountered invalid character");
                self.next()
            },
        }
    }
}

impl<'a, L: 'a + Clone + Iterator<Item = &'a str>> Iterator for Lexer<'a, L> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.start = self.range.end;

        if let Some(c) = self.next_char() {
            self._next(c)
        } else {
            None
        }
    }
}

impl<'a, L: 'a + Clone + Iterator<Item = &'a str>> core::iter::FusedIterator for Lexer<'a, L> {}
