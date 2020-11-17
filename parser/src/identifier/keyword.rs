use crate::util::ws;
use crate::Span;
///
/// Keywords
///
/// Solar contains several keywords.
use nom::IResult;

pub fn is_keyword(input: &str) -> bool {
    vec![
        "let", "set", "get", "for", "while", "if", "else", "then", "function", "return", "break",
        "match", "type", "is", "or", "in", "const", //not yet planned to be used after this
        "async",
    ]
    .contains(&input)
}

fn key(word: &'static str) -> impl Fn(Span) -> IResult<Span, Span> {
    use nom::character::complete::one_of;
    use nom::{bytes::complete::tag, sequence::delimited};
    move |s| delimited(ws, tag(word), one_of(" \n\r\t"))(s)
}

pub fn key_function(s: Span) -> IResult<Span, Span> {
    key("function")(s)
}

pub fn key_let(s: Span) -> IResult<Span, Span> {
    key("let")(s)
}
