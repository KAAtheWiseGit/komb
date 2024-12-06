#![no_std]

pub mod combinator;
mod select;
mod span;
pub mod string;

pub use span::Span;

pub type PResult<'a, I, O, E> = Result<(O, &'a I), E>;

pub trait Parser<'a, I, O, E>
where
	I: 'a + ?Sized,
{
	fn parse(&self, input: &'a I) -> PResult<'a, I, O, E>;

	fn into<OX, EX>(self) -> impl Parser<'a, I, OX, EX>
	where
		Self: Sized,
		O: Into<OX>,
		E: Into<EX>,
	{
		move |input: &'a I| match self.parse(input) {
			Ok((out, rest)) => Ok((out.into(), rest)),
			Err(err) => Err(err.into()),
		}
	}

	fn map_out<OX, F>(self, f: F) -> impl Parser<'a, I, OX, E>
	where
		Self: Sized,
		F: Fn(O) -> OX,
	{
		move |input: &'a I| {
			self.parse(input).map(|(out, rest)| (f(out), rest))
		}
	}

	fn map_err<EX, F>(self, f: F) -> impl Parser<'a, I, O, EX>
	where
		Self: Sized,
		F: Fn(E) -> EX,
	{
		move |input: &'a I| self.parse(input).map_err(&f)
	}
}

impl<'a, I, O, E, F> Parser<'a, I, O, E> for F
where
	I: 'a + ?Sized,
	F: Fn(&'a I) -> PResult<'a, I, O, E>,
{
	fn parse(&self, input: &'a I) -> PResult<'a, I, O, E> {
		self(input)
	}
}

impl<'a, I, O, E, P0, P1> Parser<'a, I, O, E> for (P0, P1)
where
	I: 'a + ?Sized,
	P0: Parser<'a, I, O, E>,
	P1: Parser<'a, I, O, E>,
{
	fn parse(&self, input: &'a I) -> PResult<'a, I, O, E> {
		if let Ok((out, rest)) = self.0.parse(input) {
			return Ok((out, rest));
		};

		match self.1.parse(input) {
			Ok((out, rest)) => Ok((out, rest)),
			Err(err) => Err(err),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_macro() {
		use string::char;

		let parser = (char('a'), char('b'));
		let result = parser.parse("bc");
		assert_eq!(Ok(('b', "c")), result);
	}
}
