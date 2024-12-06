use crate::{PResult, Parser};

macro_rules! impl_parses {
	($($type:ident: $index:tt),*; $lastp:ident: $lasti:tt) => {
		impl<'a, I, O, E, $($type,)* $lastp> Parser<'a, I, O, E> for ($($type,)*$lastp)
		where
			I: 'a + ?Sized,
			$($type: Parser<'a, I, O, E>,)*
			$lastp: Parser<'a, I, O, E>,
		{
			fn parse(&self, input: &'a I) -> PResult<'a, I, O, E> {
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

impl_parses!(P0: 0; P1: 1);
impl_parses!(P0: 0, P1: 1; P2: 2);
impl_parses!(P0: 0, P1: 1, P2: 2; P3: 3);

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_macro() {
		use crate::string::char;

		let parser = (char('a'), char('b'));
		let result = parser.parse("bc");
		assert_eq!(Ok(('b', "c")), result);

		let parser = (char('a'), char('b'), char('c'));
		let result = parser.parse("cx");
		assert_eq!(Ok(('c', "x")), result);
	}
}
