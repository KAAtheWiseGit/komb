use crate::PResult;

pub fn take_while<F>(f: F) -> impl Fn(&str) -> PResult<&str, &str, ()>
where
	F: Fn(char) -> bool,
{
	move |input: &str| {
		let mut end = 0;

		for (i, char) in input.char_indices() {
			if !f(char) {
				end = i;
				break;
			}
		}

		Ok((&input[end..], &input[..end]))
	}
}

pub fn take_until<F>(f: F) -> impl Fn(&str) -> PResult<&str, &str, ()>
where
	F: Fn(char) -> bool,
{
	take_while(move |c| !f(c))
}

pub fn one_of(
	chars: &[char],
) -> impl Fn(&str) -> PResult<&str, &str, ()> + use<'_> {
	take_while(move |c| chars.contains(&c))
}

pub fn none_of(
	chars: &[char],
) -> impl Fn(&str) -> PResult<&str, &str, ()> + use<'_> {
	take_until(move |c| chars.contains(&c))
}

pub fn whitespace() -> impl Fn(&str) -> PResult<&str, &str, ()> {
	take_while(|c| c.is_whitespace())
}

pub fn alphanumeric() -> impl Fn(&str) -> PResult<&str, &str, ()> {
	take_while(|c| c.is_alphanumeric())
}

pub fn alphabetic() -> impl Fn(&str) -> PResult<&str, &str, ()> {
	take_while(|c| c.is_alphabetic())
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn playground() {
		println!("{:?}", none_of(&['c'])("abcd"));
		println!("{:?}", one_of(&['a', 'b', 'd'])("abcd"));
	}
}
