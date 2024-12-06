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
		move |input: &'a I| self.parse(input).map_err(|err| f(err))
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
