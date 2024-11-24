use crate::Parser;

pub fn take_while<'a, F, const MIN: usize>(
	f: F,
) -> impl Parser<'a, str, &'a str, ()>
where
	F: Fn(char) -> bool,
{
	move |input: &'a str| {
		for (i, char) in input.char_indices() {
			if !f(char) {
				if input[..i].len() < MIN {
					return Err(());
				}
				return Ok((&input[i..], &input[..i]));
			}
		}

		Ok(("", input))
	}
}

pub fn take_until<'a, F, const MIN: usize>(
	f: F,
) -> impl Parser<'a, str, &'a str, ()>
where
	F: Fn(char) -> bool,
{
	take_while::<'a, _, MIN>(move |c| !f(c))
}

pub fn one_of<'a>(
	chars: &[char],
) -> impl Parser<'a, str, &'a str, ()> + use<'_, 'a> {
	take_while::<_, 0>(move |c| chars.contains(&c))
}

pub fn none_of<'a>(
	chars: &[char],
) -> impl Parser<'a, str, &'a str, ()> + use<'_, 'a> {
	take_until::<_, 0>(move |c| chars.contains(&c))
}

pub fn whitespace<'a>() -> impl Parser<'a, str, &'a str, ()> {
	take_while::<_, 0>(|c| c.is_whitespace())
}

pub fn alphanumeric<'a>() -> impl Parser<'a, str, &'a str, ()> {
	take_while::<_, 0>(|c| c.is_alphanumeric())
}

pub fn alphabetic<'a>() -> impl Parser<'a, str, &'a str, ()> {
	take_while::<_, 0>(|c| c.is_alphabetic())
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn playground() {
		assert_eq!(
			("cd", "ab"),
			none_of(&['c']).parse("abcd").unwrap()
		);
		assert_eq!(
			("cd", "ab"),
			one_of(&['a', 'b']).parse("abcd").unwrap()
		);
	}
}
