use crate::{PResult, Parser};

macro_rules! impl_tuple {
	($($type:ident $o:ident $index:tt),*) => {
		impl <'a, I, E, $($type, $o,)*> Parser<'a, I, ($($o,)*), E> for ($($type,)*)
		where
			I: 'a + ?Sized,
			$($type: Parser<'a, I, $o, E>,)*
		{
			fn parse(&self, input: &'a I)
				-> PResult<'a, I, ($($o,)*), E> {

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

impl_tuple!(P0 O0 0, P1 O1 1);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2, P3 O3 3);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2, P3 O3 3, P4 O4 4);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2, P3 O3 3, P4 O4 4, P5 O5 5);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2, P3 O3 3, P4 O4 4, P5 O5 5, P6 O6 6);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2, P3 O3 3, P4 O4 4, P5 O5 5, P6 O6 6, P7 O7 7);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2, P3 O3 3, P4 O4 4, P5 O5 5, P6 O6 6, P7 O7 7, P8 O8 8);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2, P3 O3 3, P4 O4 4, P5 O5 5, P6 O6 6, P7 O7 7, P8 O8 8, P9 O9 9);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2, P3 O3 3, P4 O4 4, P5 O5 5, P6 O6 6, P7 O7 7, P8 O8 8, P9 O9 9, P10 O10 10);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2, P3 O3 3, P4 O4 4, P5 O5 5, P6 O6 6, P7 O7 7, P8 O8 8, P9 O9 9, P10 O10 10, P11 O11 11);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2, P3 O3 3, P4 O4 4, P5 O5 5, P6 O6 6, P7 O7 7, P8 O8 8, P9 O9 9, P10 O10 10, P11 O11 11, P12 O12 12);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2, P3 O3 3, P4 O4 4, P5 O5 5, P6 O6 6, P7 O7 7, P8 O8 8, P9 O9 9, P10 O10 10, P11 O11 11, P12 O12 12, P13 O13 13);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2, P3 O3 3, P4 O4 4, P5 O5 5, P6 O6 6, P7 O7 7, P8 O8 8, P9 O9 9, P10 O10 10, P11 O11 11, P12 O12 12, P13 O13 13, P14 O14 14);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2, P3 O3 3, P4 O4 4, P5 O5 5, P6 O6 6, P7 O7 7, P8 O8 8, P9 O9 9, P10 O10 10, P11 O11 11, P12 O12 12, P13 O13 13, P14 O14 14, P15 O15 15);
impl_tuple!(P0 O0 0, P1 O1 1, P2 O2 2, P3 O3 3, P4 O4 4, P5 O5 5, P6 O6 6, P7 O7 7, P8 O8 8, P9 O9 9, P10 O10 10, P11 O11 11, P12 O12 12, P13 O13 13, P14 O14 14, P15 O15 15, P16 O16 16);

#[cfg(test)]
mod test {
	use super::*;
	use crate::string::any_char;

	#[test]
	fn basic() {
		let p = ("a", "b", "c");
		assert_eq!(Ok((("a", "b", "c"), " rest")), p.parse("abc rest"));
	}

	#[test]
	fn types() {
		let p = ("a", any_char(), "c");
		assert_eq!(Ok((("a", 'b', "c"), " rest")), p.parse("abc rest"));
	}
}
