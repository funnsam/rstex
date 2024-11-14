mod layout;
mod lexer;

pub fn render_as_html(tex: &str) -> String {
    let mut lexer = lexer::Lexer::new(tex);
    while let Some(t) = lexer.next() {
        eprintln!("{t:?}");
    }

    String::new()
}
