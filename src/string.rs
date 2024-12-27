//! Concrete parsers which operate on `str` input.
//!
//! All of the parsers return [`StringError`] for easier compositon.

use thiserror::Error;

use core::num::ParseIntError;

use crate::{combinator::choice, PResult, Parser};

/// A unified error types for all `str` parsers.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum StringError {
	/// Indicates that the input string has ended, running out of content.
	///
	/// It's most often returned when applying combinators to an empty
	/// string and in a few rare cases like [`take`].  Most other
	/// implementations, including the parser implementation on `str`,
	/// return [`Unmatched`][`StringError::Unmatched`] on length mismatch.
	#[error("reached the end of the input string")]
	End,
	/// A kitchen-sink for all kinds of parser failures.  In general, it
	/// means that a non-empty string didn't match the parser.  It doesn't
	/// apply to higher level transformations, such as integer parsing,
	/// though.
	#[error("parser failed to match")]
	Unmatched,
	/// The parser faild to convert a string to an integer.  This wraps the
	/// [`ParseIntError`] returned by `from_str_radix`.
	#[error("failed to parse integer: {0}")]
	ParseInt(ParseIntError),
}

impl<'a> Parser<'a, str, &'a str, StringError> for &str {
	fn parse(
		&self,
		input: &'a str,
	) -> PResult<'a, str, &'a str, StringError> {
		if input.starts_with(self) {
			let length = self.len();
			Ok((&input[..length], &input[length..]))
		} else if input.is_empty() {
			Err(StringError::End)
		} else {
			Err(StringError::Unmatched)
		}
	}
}

impl<'a> Parser<'a, str, char, StringError> for char {
	fn parse(&self, input: &'a str) -> PResult<'a, str, char, StringError> {
		char(move |ch| ch == *self).parse(input)
	}
}

/// Matches a `literal` ignoring the case.  This function is ASCII-only, all
/// other Unicode characters won't be accounted for.
///
/// ```rust
/// use komb::{Parser, string::{anycase, StringError}};
///
/// let p = anycase("select");
///
/// assert_eq!(Ok(("select", " from table")), p.parse("select from table"));
/// assert_eq!(Ok(("SELECT", " FROM table")), p.parse("SELECT FROM table"));
///
/// let p = anycase("löve2d");
/// assert_eq!(Err(StringError::Unmatched), p.parse("LÖVE2D"));
/// ```
pub fn anycase<'a>(
	literal: &'static str,
) -> impl Parser<'a, str, &'a str, StringError> {
	move |input: &'a str| {
		let length = literal.len();

		let mut literal_chars = literal.chars();
		let mut input_chars = input.chars();

		loop {
			let Some(lit_ch) = literal_chars.next() else {
				return Ok((
					&input[..length],
					&input[length..],
				));
			};

			let Some(input_ch) = input_chars.next() else {
				return Err(StringError::End);
			};

			if lit_ch.to_ascii_lowercase()
				!= input_ch.to_ascii_lowercase()
			{
				return Err(StringError::Unmatched);
			}
		}
	}
}

/// Matches either a `\n` or `\r\n` line ending, returns it as an `&str`
/// reference.
///
/// ```rust
/// use komb::{Parser, string::{line_end, alphanumeric0, StringError}};
///
/// let p = alphanumeric0().before(line_end());
///
/// assert_eq!(Ok(("Hello", "world")), p.parse("Hello\nworld"));
/// ```
pub fn line_end<'a>() -> impl Parser<'a, str, &'a str, StringError> {
	choice(("\n", "\r\n"))
}

/// Succeeds if the input is empty.
///
/// ```rust
/// use komb::{Parser, string::{eof, StringError}};
///
/// let p = "Hello world".before(eof());
///
/// assert_eq!(Ok(("Hello world", "")), p.parse("Hello world"));
/// assert_eq!(Err(StringError::Unmatched), p.parse("Hello world and then some"));
/// ```
pub fn eof<'a>() -> impl Parser<'a, str, (), StringError> {
	move |input: &'a str| {
		if input.is_empty() {
			Ok(((), input))
		} else {
			Err(StringError::Unmatched)
		}
	}
}

/// Takes exactly `length` characters (not bytes) from the input.  Returns
/// [`StringError::End`] if the string isn't long enough.
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

/// Cuts off a prefix of a string for whose characters the predicate `f` returns
/// `true`.
///
/// If it hits the end of the string, the whole string will be returned and the
/// remainder will be an empty string pointing to the end of input.
///
// TODO: can this be made into a macro?
/// Note that this function will succeed even if it matches no characters.  In
/// this case it'll return an empty string pointing to the start of input as the
/// output.  To fail in such cases use [`take_while1`].
///
/// ```rust
/// use komb::{Parser, string::{take_while0, StringError}};
///
/// let p = take_while0(|ch| ch.is_alphanumeric());
/// assert_eq!(Ok(("abc1", " and rest")), p.parse("abc1 and rest"));
/// assert_eq!(Ok(("", "-_-")), p.parse("-_-"))
/// ```
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

		Ok((&input[..index], &input[index..]))
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
		let mut index = 0;
		let mut at_least_one = false;

		for (i, char) in input.char_indices() {
			if !f(char) {
				if at_least_one {
					return Ok((&input[..i], &input[i..]));
				} else {
					return Err(StringError::Unmatched);
				}
			}
			index = i + char.len_utf8();
			at_least_one = true;
		}

		if at_least_one {
			return Ok((&input[..index], &input[index..]));
		} else {
			return Err(StringError::Unmatched);
		}
	}
}

pub fn take_until1<'a, F>(f: F) -> impl Parser<'a, str, &'a str, StringError>
where
	F: Fn(char) -> bool,
{
	take_while1(move |c| !f(c))
}

pub fn one_of1<'a>(
	chars: &[char],
) -> impl Parser<'a, str, &'a str, StringError> + use<'_, 'a> {
	take_while1(move |c| chars.contains(&c))
}

pub fn none_of1<'a>(
	chars: &[char],
) -> impl Parser<'a, str, &'a str, StringError> + use<'_, 'a> {
	take_until1(move |c| chars.contains(&c))
}

pub fn whitespace1<'a>() -> impl Parser<'a, str, &'a str, StringError> {
	take_while1(|c| c.is_whitespace())
}

pub fn alphanumeric1<'a>() -> impl Parser<'a, str, &'a str, StringError> {
	take_while1(|c| c.is_alphanumeric())
}

pub fn alphabetic1<'a>() -> impl Parser<'a, str, &'a str, StringError> {
	take_while1(|c| c.is_alphabetic())
}

// Character combinators

/// Returns the first character in input if it satisfies the predicate.  If the
/// predicate fails, [`StringError::Unmatched`] is returned.  If the string is
/// empty, [`StringError::End`] is returned.
///
/// ```rust
/// use komb::{Parser, string::{char, StringError}};
///
/// let p = char(|ch| ch == '1' || ch == 'a');
///
/// assert_eq!(Ok(('1', "rest")), p.parse("1rest"));
/// assert_eq!(Ok(('a', "1")), p.parse("a1"));
/// assert_eq!(Err(StringError::Unmatched), p.parse("x"));
/// ```
pub fn char<'a, F>(f: F) -> impl Parser<'a, str, char, StringError>
where
	F: Fn(char) -> bool,
{
	move |input: &'a str| {
		let Some(ch) = input.chars().next() else {
			return Err(StringError::End);
		};

		if f(ch) {
			Ok((ch, &input[ch.len_utf8()..]))
		} else {
			Err(StringError::Unmatched)
		}
	}
}

/// Returns whatever char is first in input.  It can return [`StringError::End`]
/// if the input is empty.
pub fn any_char<'a>() -> impl Parser<'a, str, char, StringError> {
	char(|_| true)
}

/// Returns the first input char if it's one of `chars`.
pub fn one_of_char<'a>(
	chars: &[char],
) -> impl Parser<'a, str, char, StringError> + use<'_, 'a> {
	char(|ch| chars.contains(&ch))
}

/// Returns the first input char if it's *not* one of `chars`.
pub fn none_of_char<'a>(
	chars: &[char],
) -> impl Parser<'a, str, char, StringError> + use<'_, 'a> {
	char(|ch| !chars.contains(&ch))
}

pub fn digits1<'a>(radix: u32) -> impl Parser<'a, str, &'a str, StringError> {
	take_while1(move |c| c.is_digit(radix))
}

macro_rules! impl_parse_uint {
	($name:ident, $type:ty) => {
		pub fn $name<'a>(
		) -> impl Parser<'a, str, ($type, &'a str), StringError> {
			|input: &'a str| {
				let (s, rest) = digits1(10)
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
