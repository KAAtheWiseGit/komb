use thiserror::Error;

use core::num::ParseIntError;

use crate::Parser;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum StringError {
	#[error("reached the end of the input string")]
	End,
	#[error("parser failed to match")]
	Unmatched,
	#[error("failed to parse integer: {0}")]
	ParseInt(ParseIntError),
}

pub fn literal<'a>(
	literal: &'static str,
) -> impl Parser<'a, str, &'a str, StringError> {
	move |input: &'a str| {
		if input.starts_with(literal) {
			let length = literal.len();
			Ok((&input[..length], &input[length..]))
		} else if input.is_empty() {
			Err(StringError::End)
		} else {
			Err(StringError::Unmatched)
		}
	}
}

pub fn line_end<'a>() -> impl Parser<'a, str, &'a str, StringError> {
	(literal("\n"), literal("\r\n"))
}

pub fn take<'a>(length: usize) -> impl Parser<'a, str, &'a str, StringError> {
	move |input: &'a str| {
		let mut current_length = 0;
		for (i, _) in input.char_indices() {
			current_length += 1;
			if current_length == length {
				return Ok((&input[..i], &input[i..]));
			}
		}

		Err(StringError::End)
	}
}

pub fn char<'a>(c: char) -> impl Parser<'a, str, char, StringError> {
	move |input: &'a str| {
		if let Some(first_char) = input.chars().next() {
			if first_char == c {
				let length = c.len_utf8();
				Ok((c, &input[length..]))
			} else {
				Err(StringError::Unmatched)
			}
		} else {
			Err(StringError::End)
		}
	}
}

pub fn take_while0<'a, F>(f: F) -> impl Parser<'a, str, &'a str, StringError>
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

pub fn take_until0<'a, F>(f: F) -> impl Parser<'a, str, &'a str, StringError>
where
	F: Fn(char) -> bool,
{
	take_while0(move |c| !f(c))
}

pub fn one_of0<'a>(
	chars: &[char],
) -> impl Parser<'a, str, &'a str, StringError> + use<'_, 'a> {
	take_while0(move |c| chars.contains(&c))
}

pub fn none_of0<'a>(
	chars: &[char],
) -> impl Parser<'a, str, &'a str, StringError> + use<'_, 'a> {
	take_until0(move |c| chars.contains(&c))
}

pub fn whitespace0<'a>() -> impl Parser<'a, str, &'a str, StringError> {
	take_while0(|c| c.is_whitespace())
}

pub fn alphanumeric0<'a>() -> impl Parser<'a, str, &'a str, StringError> {
	take_while0(|c| c.is_alphanumeric())
}

pub fn alphabetic0<'a>() -> impl Parser<'a, str, &'a str, StringError> {
	take_while0(|c| c.is_alphabetic())
}

pub fn take_while1<'a, F>(f: F) -> impl Parser<'a, str, &'a str, StringError>
where
	F: Fn(char) -> bool,
{
	move |input: &'a str| {
		let Some(ch) = input.chars().next() else {
			return Err(StringError::End);
		};

		if f(ch) {
			return Err(StringError::Unmatched);
		}

		Ok(take_while0(&f).parse(input).unwrap())
	}
}

pub fn digits<'a>(radix: u32) -> impl Parser<'a, str, &'a str, StringError> {
	take_while1(move |c| c.is_digit(radix))
}

macro_rules! impl_parse_uint {
	($name:ident, $type:ty) => {
		pub fn $name<'a>(
		) -> impl Parser<'a, str, ($type, &'a str), StringError> {
			|input: &'a str| {
				let (s, rest) = digits(10)
					.parse(input)
					.map_err(|_| {
						if input.chars()
							.next()
							.is_some()
						{
							StringError::End
						} else {
							StringError::Unmatched
						}
					})?;
				let out = s
					.parse()
					.map_err(StringError::ParseInt)?;

				Ok(((out, s), rest))
			}
		}
	};
}

impl_parse_uint!(parse_u8, u8);
impl_parse_uint!(parse_u16, u16);
impl_parse_uint!(parse_u32, u32);
impl_parse_uint!(parse_u64, u64);
impl_parse_uint!(parse_usize, usize);

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
