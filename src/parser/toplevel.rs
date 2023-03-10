use crate::{
    ast::{Function, GenericArgument},
    parser::ty::parse_type,
};

use super::{statement::parse_statement, token::*, util::*, *};

use nom::{
    branch::permutation,
    character::complete::{multispace0, space0},
    combinator::{map, opt},
    error::context,
    multi::separated_list0,
    sequence::delimited,
};

fn parse_function_decl(input: Span) -> ParseResult<FunctionDecl> {
    fn parse_generic_argument(input: Span) -> ParseResult<GenericArgument> {
        located(context(
            "generic_argument",
            map(parse_identifier, |name| GenericArgument { name }),
        ))(input)
    }
    fn parse_generic_arguments<'a>(
        input: Span<'a>,
    ) -> NotLocatedParseResult<Vec<Located<GenericArgument>>> {
        delimited(
            langlebracket,
            separated_list0(comma, parse_generic_argument),
            ranglebracket,
        )(input)
    }
    context(
        "function_decl",
        located(map(
            permutation((
                fn_token,
                delimited(multispace0, parse_identifier, multispace0),
                opt(parse_generic_arguments),
                // params
                delimited(
                    token::lparen,
                    delimited(
                        multispace0,
                        context(
                            "parameters",
                            separated_list0(
                                comma,
                                map(
                                    permutation((
                                        parse_identifier,
                                        skip0,
                                        colon,
                                        skip0,
                                        parse_type,
                                    )),
                                    |(name, _, _, _, ty)| (ty, name),
                                ),
                            ),
                        ),
                        multispace0,
                    ),
                    token::rparen,
                ),
                map(
                    permutation((space0, colon, space0, parse_type)),
                    |(_, _, _, ty)| ty,
                ),
            )),
            |(_, name, generic_args, params, ty)| FunctionDecl {
                name,
                generic_args,
                params,
                return_type: ty,
            },
        )),
    )(input)
}

pub fn parse_block(input: Span) -> NotLocatedParseResult<Vec<Located<Statement>>> {
    let (s, _) = skip0(input)?;
    let (s, _) = lbracket(s)?;
    let (s, _) = skip0(s)?;
    let mut statements = Vec::new();
    let mut s = s;
    while !s.starts_with("}") {
        let (rest, stmt) = parse_statement(s)?;
        statements.push(stmt);
        (s, _) = skip0(rest)?;
    }
    Ok((s, statements))
}

fn parse_function(input: Span) -> ParseResult<TopLevel> {
    located(context(
        "function",
        map(
            permutation((parse_function_decl, skip0, parse_block)),
            |(decl, _, body)| {
                TopLevel::Function(Function {
                    decl: decl.value,
                    body,
                })
            },
        ),
    ))(input)
}

pub(crate) fn parse_toplevel(input: Span) -> ParseResult<TopLevel> {
    context("toplevel", parse_function)(input)
}
