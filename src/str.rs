use crate::PResult;

pub fn take_while<F, const MIN: usize>(
	f: F,
) -> impl Fn(&str) -> PResult<&str, &str, ()>
where
	F: Fn(char) -> bool,
{
	move |input: &str| {
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

pub fn take_until<F, const MIN: usize>(
	f: F,
) -> impl Fn(&str) -> PResult<&str, &str, ()>
where
	F: Fn(char) -> bool,
{
	take_while::<_, MIN>(move |c| !f(c))
}

pub fn one_of(
	chars: &[char],
) -> impl Fn(&str) -> PResult<&str, &str, ()> + use<'_> {
	take_while::<_, 0>(move |c| chars.contains(&c))
}

pub fn none_of(
	chars: &[char],
) -> impl Fn(&str) -> PResult<&str, &str, ()> + use<'_> {
	take_until::<_, 0>(move |c| chars.contains(&c))
}

pub fn whitespace() -> impl Fn(&str) -> PResult<&str, &str, ()> {
	take_while::<_, 0>(|c| c.is_whitespace())
}

pub fn alphanumeric() -> impl Fn(&str) -> PResult<&str, &str, ()> {
	take_while::<_, 0>(|c| c.is_alphanumeric())
}

pub fn alphabetic() -> impl Fn(&str) -> PResult<&str, &str, ()> {
	take_while::<_, 0>(|c| c.is_alphabetic())
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn playground() {
		assert_eq!(("cd", "ab"), none_of(&['c'])("abcd").unwrap());
		assert_eq!(("cd", "ab"), one_of(&['a', 'b'])("abcd").unwrap());
	}
}
