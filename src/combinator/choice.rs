use crate::{PResult, Parser};

pub trait Choice<I, O, E> {
	fn parse(&self, input: I) -> PResult<I, O, E>;
}

/// Picks the first succeeding parser and returns it's output.  If all parsers
/// fail, the error from the last one is returned.
///
/// ```rust
/// use komb::Parse;
/// use komb::combinator::choice;
/// use komb::string::literal;
///
/// let p = choice((literal("a"), literal("b"), literal("c")));
/// assert_eq!(Ok(("a", " rest")), p.parse("a rest"));
/// assert_eq!(Ok(("b", " rest")), p.parse("b rest"));
/// assert_eq!(Ok(("c", " rest")), p.parse("c rest"));
/// assert!(p.parse("d").is_err());
/// ```
pub fn choice<'p, P, I, O, E>(parsers: P) -> Parser<'p, I, O, E>
where
	P: 'p + Choice<I, O, E>,
{
	let f = move |input| parsers.parse(input);
	Parser::from(f)
}

macro_rules! to_type {
	($t:tt) => {
		Parser<'_, I, O, E>
	}
}

macro_rules! impl_choice {
	($($index:tt),*; $lasti:tt) => {

	impl<I, O, E> Choice<I, O, E> for ($(to_type!($index),)* Parser<'_, I, O, E>)
	where
		I: Copy,
	{
		fn parse(&self, input: I) -> PResult<I, O, E> {
			$(
			if let Ok((out, rest)) = self.$index.parse(input) {
				return Ok((out, rest));
			};
			)*

			match self.$lasti.parse(input) {
				Ok((out, rest)) => Ok((out, rest)),
				Err(err) => Err(err),
			}
		}
	}

	};
}

impl_choice!(0; 1);
impl_choice!(0, 1; 2);
impl_choice!(0, 1, 2; 3);
impl_choice!(0, 1, 2, 3; 4);
impl_choice!(0, 1, 2, 3, 4; 5);
impl_choice!(0, 1, 2, 3, 4, 5; 6);
impl_choice!(0, 1, 2, 3, 4, 5, 6; 7);
impl_choice!(0, 1, 2, 3, 4, 5, 6, 7; 8);
impl_choice!(0, 1, 2, 3, 4, 5, 6, 7, 8; 9);
impl_choice!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9; 10);
impl_choice!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10; 11);
impl_choice!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11; 12);
impl_choice!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12; 13);
impl_choice!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13; 14);
impl_choice!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14; 15);

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_macro() {
		use crate::string::literal;

		let parser = choice((literal("a"), literal("b")));
		let result = parser.parse("bc");
		assert_eq!(Ok(("b", "c")), result);

		let parser = choice((literal("a"), literal("b"), literal("c")));
		let result = parser.parse("cx");
		assert_eq!(Ok(("c", "x")), result);
	}
}
