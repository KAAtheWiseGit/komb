use crate::{PResult, Parser};

pub trait Choice<'a, I, O, E>
where
	I: 'a + ?Sized,
{
	fn choice(&self, input: &'a I) -> PResult<'a, I, O, E>;
}

pub fn choice<'a, I, O, E, P>(parsers: P) -> impl Parser<'a, I, O, E>
where
	I: 'a + ?Sized,
	P: Choice<'a, I, O, E>,
{
	move |input: &'a I| parsers.choice(input)
}

macro_rules! impl_choice {
	($($type:ident: $index:tt),*; $lastp:ident: $lasti:tt) => {
		impl<'a, I, O, E, $($type,)* $lastp> Choice<'a, I, O, E> for ($($type,)*$lastp)
		where
			I: 'a + ?Sized,
			$($type: Parser<'a, I, O, E>,)*
			$lastp: Parser<'a, I, O, E>,
		{
			fn choice(&self, input: &'a I) -> PResult<'a, I, O, E> {
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

impl_choice!(P0: 0; P1: 1);
impl_choice!(P0: 0, P1: 1; P2: 2);
impl_choice!(P0: 0, P1: 1, P2: 2; P3: 3);

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_macro() {
		use crate::string::char;

		let parser = choice((char('a'), char('b')));
		let result = parser.parse("bc");
		assert_eq!(Ok(('b', "c")), result);

		let parser = choice((char('a'), char('b'), char('c')));
		let result = parser.parse("cx");
		assert_eq!(Ok(('c', "x")), result);
	}
}
