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

pub fn value<'a, I, O, E, V, P>(value: V, parser: P) -> impl Parser<'a, I, V, E>
where
	I: 'a,
	P: Parser<'a, I, O, E>,
	V: Clone,
{
	parser.map_out(move |_| value.clone())
}

/// Swaps the parser results: if the underlying parser succeeds, `not` will
/// return the output wrapped in `Err`.  If it fails, `not` parser will return
/// `Ok` with the error and the same input which was passed to it.
pub fn not<'a, I, O, E, P>(parser: P) -> impl Parser<'a, I, E, O>
where
	I: 'a,
	P: Parser<'a, I, O, E>,
{
	move |input: &'a I| match parser.parse(input) {
		Ok((out, _)) => Err(out),
		Err(err) => Ok((err, input)),
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::string::*;

	#[test]
	fn playground() {
		let res = option(take_while1(|c| c.is_ascii_lowercase()));

		assert_eq!(Ok((None, "ABCD")), res.parse("ABCD"));
	}
}
