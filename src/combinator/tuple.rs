use crate::{PResult, Parser};

pub trait Tuple<I, O> {
	fn parse(&self, input: I) -> PResult<I, O>;
}

pub fn tuple<'p, P, I, O>(parsers: P) -> Parser<'p, I, O>
where
	P: 'p + Tuple<I, O>,
{
	let f = move |input| parsers.parse(input);
	Parser::from(f)
}

// TODO: deduplicate
macro_rules! to_type {
	($o:ident) => {
		Parser<'_, I, $o>
	}
}

macro_rules! impl_tuple {
	($($o:ident $index:tt),*) => {

	impl <'a, I, $($o,)*> Tuple<I, ($($o,)*)> for ($(to_type!($o),)*)
	where
		I: Copy,
	{
		fn parse(&self, input: I) -> PResult<I, ($($o,)*)> {
			// This is an ugly, ugly hack.  The tuple
			// expressions should evaluate their arguments
			// left to right, which allows us to modify
			// `rest` on the fly.
			let mut rest = input;
			Ok((
			($({
				let (o, r) = self.$index.parse(rest)?;
				rest = r;
				o

			},)*),
			rest,
			))
		}
	}

	}
}

impl_tuple!(O0 0, O1 1);
impl_tuple!(O0 0, O1 1, O2 2);
impl_tuple!(O0 0, O1 1, O2 2, O3 3);
impl_tuple!(O0 0, O1 1, O2 2, O3 3, O4 4);
impl_tuple!(O0 0, O1 1, O2 2, O3 3, O4 4, O5 5);
impl_tuple!(O0 0, O1 1, O2 2, O3 3, O4 4, O5 5, O6 6);
impl_tuple!(O0 0, O1 1, O2 2, O3 3, O4 4, O5 5, O6 6, O7 7);
impl_tuple!(O0 0, O1 1, O2 2, O3 3, O4 4, O5 5, O6 6, O7 7, O8 8);
impl_tuple!(O0 0, O1 1, O2 2, O3 3, O4 4, O5 5, O6 6, O7 7, O8 8, O9 9);
impl_tuple!(O0 0, O1 1, O2 2, O3 3, O4 4, O5 5, O6 6, O7 7, O8 8, O9 9, O10 10);
impl_tuple!(O0 0, O1 1, O2 2, O3 3, O4 4, O5 5, O6 6, O7 7, O8 8, O9 9, O10 10, O11 11);
impl_tuple!(O0 0, O1 1, O2 2, O3 3, O4 4, O5 5, O6 6, O7 7, O8 8, O9 9, O10 10, O11 11, O12 12);
impl_tuple!(O0 0, O1 1, O2 2, O3 3, O4 4, O5 5, O6 6, O7 7, O8 8, O9 9, O10 10, O11 11, O12 12, O13 13);
impl_tuple!(O0 0, O1 1, O2 2, O3 3, O4 4, O5 5, O6 6, O7 7, O8 8, O9 9, O10 10, O11 11, O12 12, O13 13, O14 14);
impl_tuple!(O0 0, O1 1, O2 2, O3 3, O4 4, O5 5, O6 6, O7 7, O8 8, O9 9, O10 10, O11 11, O12 12, O13 13, O14 14, O15 15);

#[cfg(test)]
mod test {
	use super::*;
	use crate::string::{any_char, literal};

	#[test]
	fn basic() {
		let p = tuple((literal("a"), literal("b"), literal("c")));
		assert_eq!(Ok((("a", "b", "c"), " rest")), p.parse("abc rest"));
	}

	#[test]
	fn types() {
		let p = (literal("a"), any_char(), literal("c"));
		assert_eq!(Ok((("a", 'b', "c"), " rest")), p.parse("abc rest"));
	}
}
