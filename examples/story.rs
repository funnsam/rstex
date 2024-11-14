fn main() {
    eprintln!("{}", rstex::render_as_html(r#"
\hrule
\vskip 1in
\centerline{\bf A SHORT STORY}
\vskip 6pt
\centerline{\sl by A.~U.~Thor}
\vskip .5cm
Once upon a time, in a distant
  galaxy called \"O\"o\c c,
there lived a computer
named R.~J.~Drofnats.

Mr.~Drofnats---or \lq\lq R.~J.,\rq\rq as he preferred to be called---
was happiest when he was at work
typesetting beautiful documents.
\vskip 1in
\hrule
\vfill\eject
"#));
}
