fn main() {
    eprintln!("{}", rstex::render_as_html(r"\def\mat#1{\begin{bmatrix}#1\end{bmatrix}} % cool", true));
    eprintln!("{}", rstex::render_as_html(r"E = mc^2", true));
}
