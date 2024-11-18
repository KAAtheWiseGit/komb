use crate::Parser;

pub fn option<I, O, E, P>(parser: P) -> impl Parser<I, Option<O>, E>
where
	P: Parser<I, O, E>,
	I: Copy,
{
	move |input: I| match parser(input) {
		Ok((rest, out)) => Ok((rest, Some(out))),
		Err(_) => Ok((input, None)),
	}
}
