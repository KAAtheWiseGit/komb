use crate::PResult;

pub fn take_while<F>(input: &str, f: F) -> PResult<&str, &str, ()>
where
	F: Fn(char) -> bool,
{
	let mut end = 0;

	for (i, char) in input.char_indices() {
		if !f(char) {
			end = i;
			break;
		}
	}

	Ok((&input[end..], &input[..end]))
}

pub fn take_until<F>(input: &str, f: F) -> PResult<&str, &str, ()>
where
	F: Fn(char) -> bool,
{
	take_while(input, |c| !f(c))
}

pub fn one_of<'a>(
	input: &'a str,
	chars: &[char],
) -> PResult<&'a str, &'a str, ()> {
	take_while(input, |c| chars.contains(&c))
}

pub fn none_of<'a>(
	input: &'a str,
	chars: &[char],
) -> PResult<&'a str, &'a str, ()> {
	take_until(input, |c| chars.contains(&c))
}

pub fn whitespace(input: &str) -> PResult<&str, &str, ()> {
	take_while(input, |c| c.is_whitespace())
}

pub fn alphanumeric(input: &str) -> PResult<&str, &str, ()> {
	take_while(input, |c| c.is_alphanumeric())
}

pub fn alphabetic(input: &str) -> PResult<&str, &str, ()> {
	take_while(input, |c| c.is_alphabetic())
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn playground() {
		println!("{:?}", none_of("abcd", &['c']));
		println!("{:?}", one_of("abcd", &['a', 'b', 'd']));
	}
}
