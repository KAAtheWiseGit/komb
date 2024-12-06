use std::convert::Infallible;

use crate::Parser;

pub fn literal<'a>(literal: &'static str) -> impl Parser<'a, str, &'a str, ()> {
	move |input: &'a str| {
		if input.starts_with(literal) {
			let length = literal.len();
			Ok((&input[..length], &input[length..]))
		} else {
			Err(())
		}
	}
}

pub fn take<'a>(length: usize) -> impl Parser<'a, str, &'a str, ()> {
	move |input: &'a str| {
		let mut current_length = 0;
		for (i, _) in input.char_indices() {
			current_length += 1;
			if current_length == length {
				return Ok((&input[..i], &input[i..]));
			}
		}

		Err(())
	}
}

pub fn char<'a>(c: char) -> impl Parser<'a, str, char, ()> {
	move |input: &'a str| {
		if let Some(first_char) = input.chars().next() {
			if first_char == c {
				let length = c.len_utf8();
				Ok((c, &input[length..]))
			} else {
				Err(())
			}
		} else {
			Err(())
		}
	}
}

pub fn take_while0<'a, F>(f: F) -> impl Parser<'a, str, &'a str, Infallible>
where
	F: Fn(char) -> bool,
{
	move |input: &'a str| {
		let mut index = 0;
		for (i, char) in input.char_indices() {
			if !f(char) {
				return Ok((&input[..i], &input[i..]));
			}
			index = i + char.len_utf8();
		}

		Ok((&input[index..], &input[..index]))
	}
}

pub fn take_until0<'a, F>(f: F) -> impl Parser<'a, str, &'a str, Infallible>
where
	F: Fn(char) -> bool,
{
	take_while0(move |c| !f(c))
}

pub fn one_of0<'a>(
	chars: &[char],
) -> impl Parser<'a, str, &'a str, Infallible> + use<'_, 'a> {
	take_while0(move |c| chars.contains(&c))
}

pub fn none_of0<'a>(
	chars: &[char],
) -> impl Parser<'a, str, &'a str, Infallible> + use<'_, 'a> {
	take_until0(move |c| chars.contains(&c))
}

pub fn whitespace0<'a>() -> impl Parser<'a, str, &'a str, Infallible> {
	take_while0(|c| c.is_whitespace())
}

pub fn alphanumeric0<'a>() -> impl Parser<'a, str, &'a str, Infallible> {
	take_while0(|c| c.is_alphanumeric())
}

pub fn alphabetic0<'a>() -> impl Parser<'a, str, &'a str, Infallible> {
	take_while0(|c| c.is_alphabetic())
}

pub fn take_while1<'a, F>(f: F) -> impl Parser<'a, str, &'a str, ()>
where
	F: Fn(char) -> bool,
{
	move |input: &'a str| {
		if !input.chars().next().is_some_and(&f) {
			return Err(());
		}

		Ok(take_while0(&f).parse(input).unwrap())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn playground() {
		assert_eq!(
			("ab", "cd"),
			none_of0(&['c']).parse("abcd").unwrap()
		);
		assert_eq!(
			("ab", "cd"),
			one_of0(&['a', 'b']).parse("abcd").unwrap()
		);
	}
}
