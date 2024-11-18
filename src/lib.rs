pub type PResult<I, O, E> = Result<(I, O), E>;

pub trait Parser<I, O, E>: Fn(I) -> PResult<I, O, E> {}

impl<I, O, E, T> Parser<I, O, E> for T where T: Fn(I) -> PResult<I, O, E> {}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn playground<'a>() {
		fn sum<'a, P, T>(
			s: &'a str,
			parser: P,
		) -> PResult<&'a str, T, ()>
		where
			P: Parser<&'a str, T, ()>,
			T: std::ops::Add<Output = T>,
		{
			let (rest, a) = parser(s)?;
			let (rest, b) = parser(rest)?;

			Ok((rest, a + b))
		}

		let s = "string";

		let x = |s: &'a str| -> PResult<&'a str, u8, ()> {
			let first_char = s.as_bytes()[0];
			PResult::Ok((&s[1..], first_char))
		};

		let sum = sum("string", x).unwrap();
		println!("{:?}", sum);
	}
}
