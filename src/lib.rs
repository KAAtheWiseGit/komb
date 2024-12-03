pub mod combinator;
pub mod string;

pub type PResult<'a, I, O, E> = Result<(O, &'a I), E>;

pub trait Parser<'a, I, O, E>
where
	I: ?Sized,
{
	fn parse(&self, input: &'a I) -> PResult<'a, I, O, E>;
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
