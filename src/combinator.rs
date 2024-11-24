use crate::Parser;

pub fn option<'a, I, O, E, P>(parser: P) -> impl Parser<'a, I, Option<O>, E>
where
	I: 'a + ?Sized,
	P: Parser<'a, I, O, E>,
{
	move |input: &'a I| match parser.parse(input) {
		Ok((out, rest)) => Ok((Some(out), rest)),
		Err(_) => Ok((None, input)),
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::str::*;

	#[test]
	fn playground() {
		let res =
			option(take_while0(|c| c.is_ascii_lowercase()));

		assert_eq!(Ok((Some(""), "ABCD")), res.parse("ABCD"));
	}
}
