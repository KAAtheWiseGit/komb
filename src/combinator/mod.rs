mod choice;
pub use choice::choice;

use core::convert::Infallible;

use crate::Parser;

pub fn option<'a, I, O, E, P>(
	parser: P,
) -> impl Parser<'a, I, Option<O>, Infallible>
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

// TODO: investigate discarding the delimiting parsers errors and returning a
// custom one instead.  This will allow to mix the error types of the parsers,
// avoiding the `map_err` transforms required right now.
pub fn delimited<'a, I, E, O0, P0, O1, P1, O2, P2>(
	left: P0,
	content: P1,
	right: P2,
) -> impl Parser<'a, I, O1, E>
where
	I: 'a + ?Sized,
	P0: Parser<'a, I, O0, E>,
	P1: Parser<'a, I, O1, E>,
	P2: Parser<'a, I, O2, E>,
{
	move |input: &'a I| {
		let (_, rest) = left.parse(input)?;
		let (output, rest) = content.parse(rest)?;
		let (_, rest) = right.parse(rest)?;

		Ok((output, rest))
	}
}

pub fn fold<'a, I, O, OX, E, P, F>(
	parser: P,
	acc: OX,
	apply: F,
) -> impl Parser<'a, I, OX, E>
where
	I: 'a + ?Sized,
	P: Parser<'a, I, O, E>,
	OX: Clone,
	F: Fn(&mut OX, O),
{
	move |input: &'a I| {
		let mut acc = acc.clone();
		let mut input = input;

		loop {
			let Ok((output, rest)) = parser.parse(input) else {
				break;
			};
			input = rest;
			apply(&mut acc, output);
		}

		Ok((acc, input))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::string::*;

	#[test]
	fn playground() {
		let p = option(take_while1(|c| c.is_ascii_lowercase()));

		assert_eq!(Ok((None, "ABCD")), p.parse("ABCD"));
	}

	#[test]
	fn test_delimited() {
		let del = delimited(
			char('('),
			alphabetic0().map_err(|_| StringError::End),
			char(')'),
		);

		assert_eq!(Ok(("word", "")), del.parse("(word)"));
	}

	#[test]
	fn test_fold() {
		let p = fold(char('a'), String::new(), |acc, ch| {
			acc.push(ch);
		});

		assert_eq!(Ok((String::from("aa"), "b")), p.parse("aab"));
	}
}
