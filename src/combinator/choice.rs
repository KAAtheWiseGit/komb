use crate::{PResult, Parser};

pub struct Choice<T>(T);

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
pub fn choice<'p, P: 'p, I, O, E>(parsers: P) -> impl Parser<'p, I, O, E>
where
	Choice<P>: Parser<'p, I, O, E>,
{
	Choice(parsers)
}

macro_rules! impl_choice {
	($($p:ident $index:tt),*; $lastp:ident $lasti:tt) => {

	impl<'a, I, O, $($p,)* $lastp, E> Parser<'a, I, O, E>
		for Choice<($($p,)* $lastp)>
	where
		I: Copy,
		$($p: Parser<'a, I, O, E>,)*
		$lastp: Parser<'a, I, O, E>,
	{
		fn parse(&self, input: I) -> PResult<I, O, E> {
			$(
			if let Ok((out, rest)) = self.0.$index.parse(input) {
				return Ok((out, rest));
			};
			)*

			match self.0.$lasti.parse(input) {
				Ok((out, rest)) => Ok((out, rest)),
				Err(err) => Err(err),
			}
		}
	}

	};
}

impl_choice!(P0 0; P1 1);
impl_choice!(P0 0, P1 1; P2 2);
impl_choice!(P0 0, P1 1, P2 2; P3 3);
impl_choice!(P0 0, P1 1, P2 2, P3 3; P4 4);
impl_choice!(P0 0, P1 1, P2 2, P3 3, P4 4; P5 5);
impl_choice!(P0 0, P1 1, P2 2, P3 3, P4 4, P5 5; P6 6);
impl_choice!(P0 0, P1 1, P2 2, P3 3, P4 4, P5 5, P6 6; P7 7);
impl_choice!(P0 0, P1 1, P2 2, P3 3, P4 4, P5 5, P6 6, P7 7; P8 8);
impl_choice!(P0 0, P1 1, P2 2, P3 3, P4 4, P5 5, P6 6, P7 7, P8 8; P9 9);
impl_choice!(P0 0, P1 1, P2 2, P3 3, P4 4, P5 5, P6 6, P7 7, P8 8, P9 9; P10 10);
impl_choice!(P0 0, P1 1, P2 2, P3 3, P4 4, P5 5, P6 6, P7 7, P8 8, P9 9, P10 10; P11 11);
impl_choice!(P0 0, P1 1, P2 2, P3 3, P4 4, P5 5, P6 6, P7 7, P8 8, P9 9, P10 10, P11 11; P12 12);
impl_choice!(P0 0, P1 1, P2 2, P3 3, P4 4, P5 5, P6 6, P7 7, P8 8, P9 9, P10 10, P11 11, P12 12; P13 13);
impl_choice!(P0 0, P1 1, P2 2, P3 3, P4 4, P5 5, P6 6, P7 7, P8 8, P9 9, P10 10, P11 11, P12 12, P13 13; P14 14);
impl_choice!(P0 0, P1 1, P2 2, P3 3, P4 4, P5 5, P6 6, P7 7, P8 8, P9 9, P10 10, P11 11, P12 12, P13 13, P14 14; P15 15);

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
