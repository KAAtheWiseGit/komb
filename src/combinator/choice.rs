use crate::{PResult, Parser};

pub struct Choice<P>(P);

/// Picks the first succeeding parser and returns it's output.  If all parsers
/// fail, the error from the last one is returned.
///
/// ```rust
/// use komb::Parser;
/// use komb::combinator::choice;
///
/// let p = choice(("a", "b", "c"));
/// assert_eq!(Ok(("a", " rest")), p.parse("a rest"));
/// assert_eq!(Ok(("b", " rest")), p.parse("b rest"));
/// assert_eq!(Ok(("c", " rest")), p.parse("c rest"));
/// assert!(p.parse("d").is_err());
/// ```
pub fn choice<P>(parsers: P) -> Choice<P> {
	Choice(parsers)
}

macro_rules! impl_choice {
	($($type:ident: $index:tt),*; $lastp:ident: $lasti:tt) => {

	impl<'a, I, O, $($type,)* $lastp> Parser<'a, I, O>
		for Choice<($($type,)*$lastp)>
	where
		I: Copy,
		$($type: Parser<'a, I, O>,)*
		$lastp: Parser<'a, I, O>,
	{
		fn parse(&self, input: I) -> PResult<I, O> {
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

impl_choice!(P0: 0; P1: 1);
impl_choice!(P0: 0, P1: 1; P2: 2);
impl_choice!(P0: 0, P1: 1, P2: 2; P3: 3);
impl_choice!(P0: 0, P1: 1, P2: 2, P3: 3; P4: 4);
impl_choice!(P0: 0, P1: 1, P2: 2, P3: 3, P4: 4; P5: 5);
impl_choice!(P0: 0, P1: 1, P2: 2, P3: 3, P4: 4, P5: 5; P6: 6);
impl_choice!(P0: 0, P1: 1, P2: 2, P3: 3, P4: 4, P5: 5, P6: 6; P7: 7);
impl_choice!(P0: 0, P1: 1, P2: 2, P3: 3, P4: 4, P5: 5, P6: 6, P7: 7; P8: 8);
impl_choice!(P0: 0, P1: 1, P2: 2, P3: 3, P4: 4, P5: 5, P6: 6, P7: 7, P8: 8; P9: 9);
impl_choice!(P0: 0, P1: 1, P2: 2, P3: 3, P4: 4, P5: 5, P6: 6, P7: 7, P8: 8, P9: 9; P10: 10);
impl_choice!(P0: 0, P1: 1, P2: 2, P3: 3, P4: 4, P5: 5, P6: 6, P7: 7, P8: 8, P9: 9, P10: 10; P11: 11);
impl_choice!(P0: 0, P1: 1, P2: 2, P3: 3, P4: 4, P5: 5, P6: 6, P7: 7, P8: 8, P9: 9, P10: 10, P11: 11; P12: 12);
impl_choice!(P0: 0, P1: 1, P2: 2, P3: 3, P4: 4, P5: 5, P6: 6, P7: 7, P8: 8, P9: 9, P10: 10, P11: 11, P12: 12; P13: 13);
impl_choice!(P0: 0, P1: 1, P2: 2, P3: 3, P4: 4, P5: 5, P6: 6, P7: 7, P8: 8, P9: 9, P10: 10, P11: 11, P12: 12, P13: 13; P14: 14);
impl_choice!(P0: 0, P1: 1, P2: 2, P3: 3, P4: 4, P5: 5, P6: 6, P7: 7, P8: 8, P9: 9, P10: 10, P11: 11, P12: 12, P13: 13, P14: 14; P15: 15);
impl_choice!(P0: 0, P1: 1, P2: 2, P3: 3, P4: 4, P5: 5, P6: 6, P7: 7, P8: 8, P9: 9, P10: 10, P11: 11, P12: 12, P13: 13, P14: 14, P15: 15; P16: 16);

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_macro() {
		let parser = choice(('a', 'b'));
		let result = parser.parse("bc");
		assert_eq!(Ok(('b', "c")), result);

		let parser = choice(('a', 'b', 'c'));
		let result = parser.parse("cx");
		assert_eq!(Ok(('c', "x")), result);
	}
}
