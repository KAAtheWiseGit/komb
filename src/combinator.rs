use crate::Parser;

pub fn option<'a, I, O, E, P>(parser: P) -> impl Parser<'a, I, Option<O>, E>
where
	I: 'a + ?Sized,
	P: Parser<'a, I, O, E>,
{
	move |input: &'a I| match parser.parse(input) {
		Ok((out, rest)) => Ok((Some(out), rest)),
		Err(_) => Ok((None, input)),
	}
}

// Might be better as an implemnted method on the `Parser` trait.
pub fn into<'a, I, O1, O2, E1, E2, P, Q>(
	parser: P,
) -> impl Parser<'a, I, O2, E2>
where
	I: 'a,
	O1: Into<O2>,
	E1: Into<E2>,
	P: Parser<'a, I, O1, E1>,
{
	move |input: &'a I| match parser.parse(input) {
		Ok((out, rest)) => Ok((out.into(), rest.into())),
		Err(err) => Err(err.into()),
	}
}

pub fn map_out<'a, I, O1, O2, E, P, F>(
	parser: P,
	f: F,
) -> impl Parser<'a, I, O2, E>
where
	I: 'a,
	P: Parser<'a, I, O1, E>,
	F: Fn(O1) -> O2,
{
	move |input: &'a I| {
		parser.parse(input).map(|(out, rest)| (f(out), rest))
	}
}

pub fn value<'a, I, O, E, V, P>(value: V, parser: P) -> impl Parser<'a, I, V, E>
where
	I: 'a,
	P: Parser<'a, I, O, E>,
	V: Clone,
{
	map_out(parser, move |_| value.clone())
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::str::*;

	#[test]
	fn playground() {
		let res = option(take_while1(|c| c.is_ascii_lowercase()));

		assert_eq!(Ok((None, "ABCD")), res.parse("ABCD"));
	}
}
