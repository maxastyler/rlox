use crate::ast::{Binary, Expression, Literal, Parenthesised, Symbol};
use nom::{
    bytes::complete::{tag, take_while_m_n},
    character::{
        complete::{alpha0, alpha1, alphanumeric0, multispace0, space0, space1},
        is_alphabetic,
    },
    combinator::verify,
    error::ParseError,
    multi::{many0, many1, separated_list0},
    number,
    number::complete::double,
    sequence::preceded,
    sequence::{delimited, tuple},
    IResult, InputLength, Parser,
};

fn nil(input: &str) -> IResult<&str, Literal> {
    let (s, _) = tag("nil")(input)?;
    Ok((s, Literal::Nil))
}

fn false_lit(input: &str) -> IResult<&str, Literal> {
    let (s, _) = tag("false")(input)?;
    Ok((s, Literal::False))
}

fn true_lit(input: &str) -> IResult<&str, Literal> {
    let (s, _) = tag("true")(input)?;
    Ok((s, Literal::True))
}

fn number(input: &str) -> IResult<&str, Literal> {
    let (s, n) = double(input)?;
    Ok((s, Literal::Double(n)))
}

fn literal(input: &str) -> IResult<&str, Literal> {
    nil.or(false_lit)
        .or(true_lit)
        .or(number)
        .or(symbol.map(Literal::Symbol))
        .parse(input)
}

fn pexpression(input: &str) -> IResult<&str, Expression> {
    delimited(tag("("), expression, tag(")"))(input)
}

fn bin<'a>(input: &'a str, op: &str) -> IResult<&'a str, ()> {
    let (s, _) = delimited(multispace0, tag(op), multispace0)(input)?;
    Ok((s, ()))
}

fn bin_op<'a>(input: &'a str, operators: &[&str]) -> IResult<&'a str, Expression> {
    match operators {
        [] => pexpression
            .or(block)
            .or(call)
            .or(literal.map(Expression::Literal))
            .parse(input),
        [a, rest @ ..] => {
            let (s, first_exp) = bin_op(input, rest)?;
            let (s, rest_exps) = many0(tuple((|x| bin(x, a), |x| bin_op(x, rest))))(s)?;
            let mut v = vec![first_exp];
            v.extend(rest_exps.into_iter().map(|(_, x)| x));
            Ok((
                s,
                Expression::Binary(Box::new(Binary::from_op_string(a, v))),
            ))
        }
    }
}

fn symbol(input: &str) -> IResult<&str, Symbol> {
    let (s, f) = alpha1(input)?;
    let (s, rest) = alphanumeric0(s)?;
    Ok((s, Symbol(format!("{f}{rest}"))))
}

fn assignment(input: &str) -> IResult<&str, Expression> {
    let (s, sym) = symbol(input)?;
    let (s, _) = delimited(space0, tag("="), space0)(s)?;
    let (s, e) = expression(s)?;
    Ok((s, Expression::Assignment(sym, Box::new(e))))
}

fn arguments(input: &str) -> IResult<&str, Vec<Expression>> {
    delimited(
        tag("("),
        separated_list0(delimited(multispace0, tag(","), multispace0), expression),
        tag(")"),
    )(input)
}

fn call(input: &str) -> IResult<&str, Expression> {
    let (s, f) = symbol(input)?;
    let (s, args) = preceded(space0, arguments)(s)?;
    Ok((s, Expression::Call(f, args)))
}

fn block(input: &str) -> IResult<&str, Expression> {
    let (s, v) = delimited(
        tag("{"),
        separated_list0(delimited(multispace0, tag(";"), multispace0), expression),
        tag("}"),
    )(input)?;
    Ok((s, Expression::Block(v)))
}

fn expression(input: &str) -> IResult<&str, Expression> {
    block
        .or(pexpression)
        .or(assignment)
        .or(call)
        .or(|x| bin_op(x, Binary::ops()))
        .or(literal.map(Expression::Literal))
        .map(|x| x.simplify())
        .parse(input)
}

#[cfg(test)]
mod tests {

    use nom::multi::separated_list1;

    use super::*;
    use crate::ast::Literal;

    #[test]
    fn full_expression() {
        assert_eq!(
            expression("2+(3+x) and (x = 5) and c(x, y, z, 10, f=4, {a=1;b=3}, c=2)"),
            Ok(("", Expression::Literal(Literal::Double(3.0))))
        );
    }

    #[test]
    fn test_separated() {
        assert_eq!(
            separated_list1(
                delimited(space0, tag(";"), space0),
                assignment
                    .or(|x| bin_op(x, Binary::ops()))
                    .map(|x| x.simplify())
            )("a=2; b=3; 30"),
            Ok(("", vec![]))
        );
    }

    #[test]
    fn test_arguments() {
        assert_eq!(
            call("af4(a=3, b=4)"),
            Ok(("", Expression::Call(Symbol("hi".into()), vec![])))
        );
    }

    #[test]
    fn general_parsing_test() {
        assert_eq!(
            bin_op("1 and (3 + 4) and 2", Binary::ops()).map(|(s, b)| (s, b.simplify())),
            Ok(("", Expression::Literal(Literal::Double(0.0))))
        );
    }

    #[test]
    fn test_symbol() {
        assert_eq!(symbol("syyy"), Ok(("", Symbol("syyy".into()))));
        assert_eq!(symbol("s23!"), Ok(("!", Symbol("s23".into()))));
        assert!(symbol("31!").is_err());
    }

    #[test]
    fn test_assignment() {
        assert_eq!(
            assignment("a =    3"),
            Ok(("", Expression::Literal(Literal::Double(3.0))))
        )
    }
}
