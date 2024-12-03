use crate::{PResult, Parser};

use std::marker::PhantomData;
struct Wrapper<'a, I, O, E, P>
where
	I: 'a + ?Sized,
	P: Parser<'a, I, O, E>,
{
	inner: P,
	marker_i: PhantomData<&'a I>,
	marker_o: PhantomData<O>,
	marker_e: PhantomData<E>,
}

use std::ops::Deref;
impl<'a, I, O, E, P> Deref for Wrapper<'a, I, O, E, P>
where
	P: Parser<'a, I, O, E>,
{
	type Target = P;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

pub struct Select<T> {
	inner: T,
}

impl<'a, I, O, O0, O1, E, E0, E1, P0, P1> Parser<'a, I, O, E>
	for Select<(Wrapper<'a, I, O0, E0, P0>, Wrapper<'a, I, O1, E1, P1>)>
where
	I: 'a + ?Sized,
	P0: Parser<'a, I, O0, E0>,
	P1: Parser<'a, I, O1, E1>,
	O0: Into<O>,
	O1: Into<O>,
	E0: Into<E>,
	E1: Into<E>,
{
	fn parse(&self, input: &'a I) -> PResult<'a, I, O, E> {
		if let Ok((out, rest)) = self.inner.0.inner.parse(input) {
			return Ok((out.into(), rest));
		}

		match self.inner.1.inner.parse(input) {
			Ok((out, rest)) => Ok((out.into(), rest)),
			Err(err) => Err(err.into()),
		}
	}
}

pub fn select<'a, I, O, E, O0, O1, E0, E1, P0, P1>(
	p0: P0,
	p1: P1,
) -> impl Parser<'a, I, O, E>
where
	I: 'a + ?Sized,
	P0: Parser<'a, I, O0, E0>,
	P1: Parser<'a, I, O1, E1>,
	O0: Into<O>,
	O1: Into<O>,
	E0: Into<E>,
	E1: Into<E>,
{
	let p0 = Wrapper {
		inner: p0,
		marker_i: PhantomData,
		marker_o: PhantomData,
		marker_e: PhantomData,
	};
	let p1 = Wrapper {
		inner: p1,
		marker_i: PhantomData,
		marker_o: PhantomData,
		marker_e: PhantomData,
	};
	let parser = Select { inner: (p0, p1) };

	move |input: &'a I| parser.parse(input)
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::string::char;

	#[test]
	fn basic() {
		let parser = select::<'_, str, char, (), _, _, _, _, _, _>(
			char('a'),
			char('b'),
		);

		let result = parser.parse("bc");
		assert_eq!(Ok(('b', "c")), result);
	}
}
