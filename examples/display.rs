fn main() {
    eprintln!("{}", rstex::render_as_html(r"
\def\mat#1{\begin{bmatrix}#1\end{bmatrix}}
\mat{1 & 2 \\ 3 & 4}"));
    eprintln!("{}", rstex::render_as_html(r"$E   =   mc^2$"));

    eprintln!("{}", rstex::render_as_html(r"\TeX  is
  cool  \TeX ^^`^^` abc  ^^`^^` ab

a"));
}
