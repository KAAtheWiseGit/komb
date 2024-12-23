#![no_std]

pub mod combinator;
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

	/// Calls the `other` parser if this one fails and returns it's result
	/// instead.
	fn or<Q>(self, other: Q) -> impl Parser<'a, I, O, E>
	where
		Self: Sized,
		Q: Parser<'a, I, O, E>,
	{
		move |input: &'a I| {
			self.parse(input).or_else(|_| other.parse(input))
		}
	}

	/// Replaces the error with `default` and untouched input if the parser
	/// fails.  Similar to [`Result::or`], which it uses under the hood.
	fn or_value(self, default: O) -> impl Parser<'a, I, O, E>
	where
		Self: Sized,
		O: Clone,
	{
		move |input: &'a I| {
			self.parse(input).or(Ok((default.clone(), input)))
		}
	}

	/// If the parser succeeds, `and_then` discards the output and returns
	/// the result of the `next` parser.  If either parser fails, the error
	/// is returned immediately.
	fn and_then<Q, OX>(self, next: Q) -> impl Parser<'a, I, OX, E>
	where
		Self: Sized,
		Q: Parser<'a, I, OX, E>,
	{
		move |input: &'a I| {
			self.parse(input).and_then(|(_, rest)| next.parse(rest))
		}
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
