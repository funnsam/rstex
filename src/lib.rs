#![feature(if_let_guard)]

mod lexer;

pub fn render_as_html(tex: &str, display: bool) -> String {
    let mut lexer = lexer::Lexer::new(tex);
    while let Some(t) = lexer.next() {
        eprintln!("{t:?} {:?} {}", lexer.range(), &tex[lexer.range()]);
    }

    String::new()
}
