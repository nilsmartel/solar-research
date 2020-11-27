// Holds all information about parsing record types in solar

use crate::{generics::*, identifier::Identifier, types::Type, util::to_failure, Parse, Span};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt},
    multi::many1,
};

#[derive(Clone, Debug)]
pub struct Structure<'a> {
    pub pos: Span<'a>,
    pub public: bool,
    pub name: Identifier<'a>,
    pub generics: GenericHeader<'a>,
    pub fields: EnumOrStructFields<'a>,
}

impl<'a> Parse<'a> for Structure<'a> {
    fn parse(s: Span<'a>) -> nom::IResult<Span<'a>, Self> {
        use crate::keyword::{key_pub, key_type};

        // pub
        let (rest, public) = opt(key_pub)(s)?;
        let public = public.is_some();

        // type
        let (rest, _) = key_type(rest)?;

        // after this all errors are non recoverable
        // because no other token may be recognized

        // Node
        let (rest, name) = Identifier::parse_ws(rest).map_err(to_failure)?;
        let pos = name.pos;
        // T
        let (rest, generics) = GenericHeader::parse_ws(rest).map_err(to_failure)?;
        // - value: T
        // - next: Node T
        let (rest, fields) = EnumOrStructFields::parse_ws(rest).map_err(to_failure)?;

        Ok((
            rest,
            Structure {
                pos,
                public,
                name,
                generics,
                fields,
            },
        ))
    }
}

#[derive(Clone, Debug)]
pub enum EnumOrStructFields<'a> {
    Enum(EnumFields<'a>),
    Struct(StructFields<'a>),
}

impl<'a> Parse<'a> for EnumOrStructFields<'a> {
    fn parse(s: Span<'a>) -> nom::IResult<Span<'a>, Self> {
        alt((
            map(EnumFields::parse, EnumOrStructFields::Enum),
            map(StructFields::parse, EnumOrStructFields::Struct),
        ))(s)
    }
}

#[derive(Clone, Debug)]
pub struct EnumFields<'a> {
    pub pos: Span<'a>,
    pub states: Vec<EnumField<'a>>,
}

impl<'a> Parse<'a> for EnumFields<'a> {
    fn parse(s: Span<'a>) -> nom::IResult<Span<'a>, Self> {
        // TODO implement parsing of `= Left A | Right B`
        let (rest, states) = many1(EnumField::parse_ws)(s)?;

        Ok((
            rest,
            EnumFields {
                pos: states[0].pos,
                states,
            },
        ))
    }
}

#[derive(Clone, Debug)]
pub struct EnumField<'a> {
    pub pos: Span<'a>,
    pub name: Identifier<'a>,
    // Optional value. For now can only hold one type.
    // and now name assiciated with that field
    pub value: Option<Type<'a>>,
}

impl<'a> Parse<'a> for EnumField<'a> {
    fn parse(s: Span<'a>) -> nom::IResult<Span<'a>, Self> {
        let (rest, _) = tag("|")(s)?;
        let (rest, name) = Identifier::parse_ws(rest)?;
        let (rest, value) = opt(Type::parse_ws)(rest)?;

        Ok((
            rest,
            EnumField {
                pos: name.pos,
                name,
                value,
            },
        ))
    }
}

#[derive(Clone, Debug)]
pub struct StructFields<'a> {
    pub pos: Span<'a>,
    pub fields: Vec<StructField<'a>>,
}

// Person name="Nils" age=23 preference=(Computer os="macOS" vendor="apple)
impl<'a> Parse<'a> for StructFields<'a> {
    fn parse(s: Span<'a>) -> nom::IResult<Span<'a>, Self> {
        // let (rest, fields) = many1(StructField::parse_ws)(s)?;
        let mut input = s;
        let mut fields = Vec::new();
        loop {
            if let Ok((rest, field)) = StructField::parse_ws(input) {
                input = rest;
                fields.push(field);
            } else {
                break;
            }
        }

        let pos = fields[0].pos;

        Ok((input, StructFields { pos, fields }))
    }
}

#[derive(Clone, Debug)]
pub struct StructField<'a> {
    pub pos: Span<'a>,
    pub name: Identifier<'a>,
    pub value: Type<'a>,
}

impl<'a> Parse<'a> for StructField<'a> {
    fn parse(s: Span<'a>) -> nom::IResult<Span<'a>, Self> {
        let (rest, _) = tag("-")(s)?;

        let (rest, name) = Identifier::parse_ws(rest)?;

        let (rest, value) = Type::parse_ws(rest)?;

        Ok((
            rest,
            StructField {
                pos: name.pos,
                name,
                value,
            },
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn individual_struct_field() {
        let input = Span::from(
            "
        - value Node T",
        );

        let result = StructField::parse_ws(input);
        assert!(result.is_ok());
        let (rest, _field) = result.unwrap();
        assert_eq!(*rest, "")
    }
    #[test]
    fn parsing_entire_structs_works() {
        let input = Span::from(
            "
        pub type Node T
        - value T
        - next Node T
        ",
        );

        // let expected = {
        //     let name: Identifier = "Node".must_parse();
        //     let generics = "T".must_parse();
        //     let fields = "- value T - next Node T".must_parse();

        //     Structure {
        //         pos: name.pos,
        //         name,
        //         generics,
        //         fields,
        //         public: false,
        //     }
        // };

        let output = Structure::parse_ws(input);
        assert!(output.is_ok());
        // TODO test more extensive
    }
}
