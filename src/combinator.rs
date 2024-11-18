use crate::Parser;

pub fn option<I, O, E, P>(parser: P) -> impl Parser<I, Option<O>, E>
where
	P: Parser<I, O, E>,
{
	move |input: I| match parser(input) {
		Ok((rest, out)) => Ok((rest, Some(out))),
		Err((input, _)) => Ok((input, None)),
	}
}
