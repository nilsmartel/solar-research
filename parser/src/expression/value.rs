use nom::{
    branch::alt,
    bytes::complete::escaped_transform,
    bytes::complete::take_while_m_n,
    bytes::complete::{tag, take, take_while, take_while1},
    character::complete::none_of,
    combinator::{map, opt, recognize},
    multi::many1,
    number::complete::recognize_float,
    sequence::preceded,
    sequence::terminated,
};

use super::Expression;

use crate::{
    util::characters::{digit10, digit16, digit2, digit8},
    Parse, Span,
};

#[derive(Clone, Debug)]
pub enum Value<'a> {
    Integer(IntegerType<'a>),
    Float(FloatType<'a>),
    String(StringType<'a>),
}

#[derive(Clone, Debug)]
pub struct FloatType<'a> {
    pub span: Span<'a>,
}

impl<'a> Parse<'a> for FloatType<'a> {
    fn parse(s: Span<'a>) -> nom::IResult<Span<'a>, Self> {
        map(recognize_float, |span| FloatType { span })(s)
    }
}

#[derive(Clone, Debug)]
pub struct IntegerType<'a> {
    pub span: Span<'a>,
}

impl<'a> Parse<'a> for IntegerType<'a> {
    fn parse(s: Span<'a>) -> nom::IResult<Span<'a>, Self> {
        let recognize_int = alt((
            take_while1(digit10),
            preceded(tag("0x"), take_while1(digit16)),
            preceded(tag("0o"), take_while1(digit8)),
            preceded(tag("0b"), take_while1(digit2)),
        ));
        map(recognize_int, |span| IntegerType { span })(s)
    }
}

#[derive(Clone, Debug)]
pub struct StringType<'a> {
    pub span: Span<'a>,
    pub builder: Vec<StringBuilder<'a>>,
}

impl<'a> Parse<'a> for StringType<'a> {
    fn parse(s: Span<'a>) -> nom::IResult<Span<'a>, Self> {
        let (rest, start) = tag("\"")(s)?;
        let span = start;

        let (rest, builder) = many1(|s| parse_string_content(StringBuilder::default(), s))(rest)?;

        return Ok((rest, StringType { span, builder }));
    }
}

/// Type able to hold information about string interpolation stuff
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct StringBuilder<'a> {
    pub content: String,
    pub expression: Option<Expression<'a>>,
}

// I've written a lot of bad code, this might be the worst yet
fn parse_string_content<'a>(
    mut builder: StringBuilder<'a>,
    s: Span<'a>,
) -> nom::IResult<Span<'a>, StringBuilder<'a>> {
    let (rest, content) = take_while(|c| c != '"' && c != '\\')(s)?;

    // check if character was a " (end of string)
    if rest.as_bytes()[0] == b'"' {
        // End of string is reached
        return Ok((
            rest,
            StringBuilder {
                content: content.to_string(),
                expression: None,
            },
        ));
    }

    // push content onto string
    builder.content.push_str(*content);

    // we now know, that an \ character caused us to stop parsing
    let (rest, _) = take(1usize)(rest)?;

    match take(1usize)(rest)? {
        (rest, span) if *span == "n" => {
            builder.content.push('\n');
            parse_string_content(builder, rest)
        }
        (rest, span) if *span == "r" => {
            builder.content.push('\r');
            parse_string_content(builder, rest)
        }
        (rest, span) if *span == "t" => {
            builder.content.push('\t');
            parse_string_content(builder, rest)
        }
        (rest, span) if *span == "u" => {
            // ascii characters it is!
            let (rest, code) = take_while_m_n(2, 2, digit16)(rest)?;
            // this is ugly, ugly code
            let unichar = u64::from_str_radix(*code, 16).unwrap() as u8 as char;
            builder.content.push(unichar);
            parse_string_content(builder, rest)
        }
        (rest, span) if *span == "\"" || *span == "\\" => {
            builder.content.push_str(*span);
            parse_string_content(builder, rest)
        }
        (rest, span) if *span == "(" => {
            // String Interpolation! We now want to parse an expression
            let (rest, expression) = Expression::parse_ws(rest)?;
            let (rest, _) = tag(")")(rest)?;
            Ok((
                rest,
                StringBuilder {
                    expression: Some(expression),
                    ..builder
                },
            ))
        }
        (rest, span) => {
            // technically we'd like to error here
            builder.content.push_str(*span);
            parse_string_content(builder, rest)
        }
    }
}
