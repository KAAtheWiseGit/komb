use crate::PResult;

pub fn option<'a, I, O, E, P>(
	parser: P,
) -> impl Fn(&'a I) -> PResult<&I, Option<O>, E>
where
	I: ?Sized + 'a,
	P: Fn(&'a I) -> PResult<&I, O, E>,
{
	move |input: &I| match parser(input) {
		Ok((rest, out)) => Ok((rest, Some(out))),
		Err(_) => Ok((input, None)),
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::str::*;

	#[test]
	fn playground() {
		let res = option(alphabetic());

		println!("{:?}", res("abcd"));
	}
}
