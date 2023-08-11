use nom::{
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{multispace0, multispace1},
    combinator::opt,
    error::ParseError,
    multi::{many0, many1, separated_list0, separated_list1},
    number::complete::double,
    sequence::{delimited, tuple},
    AsChar, IResult, InputLength, InputTakeAtPosition, Parser,
};

use crate::ast::{
    Assignment, Block, Call, Cond, Expression, Function, Literal, Parenthesised, Symbol,
};

const RESERVED_STRINGS: &[&str] = &["=>", "let", "cond", "fn"];

const RESERVED_CHARS: &[char] = &['(', ')', '{', '}', '.', ';', ',', '!', '"'];

fn nil(input: &str) -> IResult<&str, Literal> {
    let (s, _) = tag("nil")(input)?;
    Ok((s, Literal::Nil))
}

fn false_lit(input: &str) -> IResult<&str, Literal> {
    let (s, _) = tag("false")(input)?;
    Ok((s, Literal::Boolean(false)))
}

fn true_lit(input: &str) -> IResult<&str, Literal> {
    let (s, _) = tag("true")(input)?;
    Ok((s, Literal::Boolean(true)))
}

fn number(input: &str) -> IResult<&str, Literal> {
    let (s, n) = double(input)?;
    Ok((s, Literal::Number(n)))
}

fn symbol(input: &str) -> IResult<&str, Symbol> {
    let (s, first) = take_while1(|x: char| {
        !(x.is_whitespace() | x.is_digit(10) | RESERVED_CHARS.iter().any(|&c| c == x))
    })(input)?;
    let (s, rest) =
        take_while(|x: char| !(x.is_whitespace() | RESERVED_CHARS.iter().any(|&c| c == x)))(s)?;
    let full_string = format!("{first}{rest}");
    if RESERVED_STRINGS.iter().any(|&x| x == full_string) {
        Err(nom::Err::Error(nom::error::Error::from_error_kind(
            input,
            nom::error::ErrorKind::Char,
        )))
    } else {
        Ok((s, Symbol(full_string)))
    }
}

fn surrounded_list<I, O, O1, O2, O3, E, F, G, H, J>(
    mut first: F,
    mut separator: G,
    mut value: H,
    mut last: J,
) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: InputTakeAtPosition + Clone + InputLength,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    H: Parser<I, O, E>,
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
    J: Parser<I, O3, E>,
    E: ParseError<I>,
{
    move |s| {
        let (s, _) = first.parse(s)?;
        let (s, _) = multispace0(s)?;
        let (s, values) = separated_list0(
            many1(delimited(multispace0, |x| separator.parse(x), multispace0)),
            |x| value.parse(x),
        )(s)?;
        let (s, _) = multispace0(s)?;
        let (s, _) = last.parse(s)?;
        Ok((s, values))
    }
}

/// Delimite the given parser by spaces
fn s_d<I, O, E, F>(p: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: InputTakeAtPosition + Clone + InputLength,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    delimited(multispace0, p, multispace0)
}

/// A function that returns a list separated and possibly delimited by the separator.
/// The separator has to occur at least once, and can occur multiple times
/// Returns the vector of values in the list, and also a boolean which is true if the
/// function ended with the separator
fn separated_delimited_list0<I, O, O2, E, F, G>(
    mut sep: G,
    mut f: F,
) -> impl FnMut(I) -> IResult<I, (Vec<O>, bool), E>
where
    I: Clone + InputLength,
    F: Parser<I, O, E>,
    G: Parser<I, O2, E>,
    E: ParseError<I>,
{
    move |s| {
        let (s, _) = opt(many1(|x| sep.parse(x)))(s)?;
        let (s, items) = separated_list0(many1(|x| sep.parse(x)), |x| f.parse(x))(s)?;
        let (s, last) = opt(many1(|x| sep.parse(x)))(s)?;
        Ok((s, (items, last.is_some())))
    }
}

fn function(input: &str) -> IResult<&str, Function> {
    let (s, _) = tag("fn")(input)?;
    let (s, fn_symbol) = s_d(opt(symbol))(s)?;
    let (s, args) = opt(delimited(
        tag("("),
        separated_delimited_list0(s_d(tag(",")), symbol),
        tag(")"),
    ))(s)?;
    let (s, _) = multispace0(s)?;
    let (s, b) = block(s)?;
    Ok((
        s,
        Function {
            identifier: fn_symbol,
            arguments: args.map(|(a, _)| a).unwrap_or_else(Vec::new),
            block: b,
        },
    ))
}

fn literal(input: &str) -> IResult<&str, Literal> {
    nil.or(number).or(true_lit).or(false_lit).parse(input)
}

fn parenthesised(input: &str) -> IResult<&str, Parenthesised> {
    let (s, exps) = delimited(tag("("), expressions, tag(")"))(input)?;
    Ok((s, Parenthesised(exps)))
}

fn block(input: &str) -> IResult<&str, Block> {
    let (s, exps) = delimited(tag("{"), expressions, tag("}"))(input)?;
    Ok((s, Block(exps)))
}

fn l1(input: &str) -> IResult<&str, Expression> {
    block
        .map(|x| Expression::Block(Box::new(x)))
        .or(parenthesised.map(|x| Expression::Parenthesised(Box::new(x))))
        .or(function.map(|x| Expression::Function(Box::new(x))))
        .or(literal.map(Expression::Literal))
        .or(symbol.map(Expression::Symbol))
        .parse(input)
}

fn no_arg_call(input: &str) -> IResult<&str, Call> {
    let (s, e) = l1(input)?;
    let (s, _) = tuple((multispace0, tag("!")))(s)?;
    Ok((
        s,
        Call {
            function: e,
            arguments: vec![],
        },
    ))
}

fn l2(input: &str) -> IResult<&str, Expression> {
    no_arg_call
        .map(|x| Expression::Call(Box::new(x)))
        .or(l1)
        .parse(input)
}

fn infix_call(input: &str) -> IResult<&str, Call> {
    let (s, arg) = l2(input)?;
    let (s, _) = s_d(tag("."))(s)?;
    let (s, fun) = l2(s)?;
    let (s, (mut other_args, _)) = separated_delimited_list0(multispace1, l2)(s)?;
    let mut args = vec![arg];
    args.append(&mut other_args);
    Ok((
        s,
        Call {
            function: fun,
            arguments: args,
        },
    ))
}

fn l3(input: &str) -> IResult<&str, Expression> {
    infix_call
        .map(|x| Expression::Call(Box::new(x)))
        .or(l2)
        .parse(input)
}

fn normal_call(input: &str) -> IResult<&str, Call> {
    let (s, f) = l3(input)?;
    let (s, _) = multispace1(s)?;
    let (s, args) = separated_list1(multispace1, l3)(s)?;
    Ok((
        s,
        Call {
            function: f,
            arguments: args,
        },
    ))
}

fn l4(input: &str) -> IResult<&str, Expression> {
    normal_call
        .map(|x| Expression::Call(Box::new(x)))
        .or(l3)
        .parse(input)
}

fn assignment(input: &str) -> IResult<&str, Assignment> {
    let (s, _) = tag("let")(input)?;
    let (s, id) = s_d(symbol)(s)?;
    let (s, ex) = expression(s)?;
    Ok((
        s,
        Assignment {
            identifier: id,
            value: ex,
        },
    ))
}

fn cond(input: &str) -> IResult<&str, Cond> {
    let (s, _) = tag("cond")(input)?;
    let (s, _) = s_d(tag("{"))(s)?;
    let (s, (exps, _)) = separated_delimited_list0(
        s_d(tag(",")),
        tuple((expression, s_d(tag("=>")), expression)).map(|(x1, _, x2)| (x1, x2)),
    )(s)?;
    let (s, last_expression) = opt(expression)(s)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("}")(s)?;
    Ok((
        s,
        Cond {
            conditions: exps,
            otherwise: last_expression,
        },
    ))
}

fn expression(input: &str) -> IResult<&str, Expression> {
    let (s, ex) = assignment
        .map(|x| Expression::Assignment(Box::new(x)))
        .or(cond.map(|x| Expression::Cond(Box::new(x))))
        .or(l4)
        .parse(input)?;
    let (s, ignored) = opt(tuple((multispace0, tag(";"))))(s)?;
    if ignored.is_some() {
        Ok((s, Expression::Ignored(Box::new(ex))))
    } else {
        Ok((s, ex))
    }
}

fn expressions(input: &str) -> IResult<&str, Vec<Expression>> {
    many0(delimited(multispace0, expression, multispace0))(input)
}

pub fn parse(input: &str) -> IResult<&str, Vec<Expression>> {
    expressions(input)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_expressions() {
        assert_eq!(
            expressions("\n\n2\n; 3.1; let x \n\n s 4"),
            Ok((
                "",
                vec![
                    Expression::Ignored(Box::new(Expression::Literal(Literal::Number(2.0)))),
                    Expression::Ignored(Box::new(Expression::Literal(Literal::Number(3.1)))),
                    Expression::Assignment(Box::new(Assignment {
                        identifier: Symbol("x".into()),
                        value: Expression::Call(Box::new(Call {
                            function: Expression::Symbol(Symbol("s".into())),
                            arguments: vec![Expression::Literal(Literal::Number(4.0))]
                        }))
                    }))
                ]
            ))
        )
    }

    #[test]
    fn test_symbol() {
        assert_eq!(
            symbol("hello_there"),
            Ok(("", Symbol("hello_there".into())))
        );
        assert_eq!(symbol("a symbol"), Ok((" symbol", Symbol("a".into()))));
        assert!(symbol("2start").is_err());
        assert!(symbol("let").is_err());
        assert_eq!(symbol("=23"), Ok(("", Symbol("=23".into()))));
        assert_eq!(symbol("=2;!3"), Ok((";!3", Symbol("=2".into()))));
    }

    #[test]
    fn test_parsing_function() {
        assert_eq!(
            function("fn x (a,b,c) {x.* a}"),
            Ok((
                "",
                Function {
                    identifier: Some(Symbol("x".into())),
                    arguments: vec![Symbol("a".into()), Symbol("b".into()), Symbol("c".into())],
                    block: Block(vec![Expression::Call(Box::new(Call {
                        function: Expression::Symbol(Symbol("*".into())),
                        arguments: vec![
                            Expression::Symbol(Symbol("x".into())),
                            Expression::Symbol(Symbol("a".into()))
                        ]
                    }))])
                }
            ))
        )
    }

    #[test]
    fn test_surrounded_list() {
        assert_eq!(
            separated_delimited_list0(delimited(multispace0, tag(","), multispace0), number)
                .parse(",, , ,,,,2,, 3, 4, 5 ,,, , ,,,"),
            Ok((
                "",
                (
                    vec![
                        Literal::Number(2.0),
                        Literal::Number(3.0),
                        Literal::Number(4.0),
                        Literal::Number(5.0)
                    ],
                    true
                )
            ))
        );
        assert_eq!(
            separated_delimited_list0(delimited(multispace0, tag(","), multispace0), number)
                .parse(",,,"),
            Ok(("", (vec![], false)))
        );
    }

    #[test]
    fn test_cond() {
        assert!(cond("cond {x => 2,    , , , 2 .+ 3 => 3,, x .* 30 }").is_ok(),)
    }
}
