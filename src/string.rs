//! Concrete parsers which operate on `str` input.
//!
//! All of the parsers return [`Error`] for easier compositon.

use crate::{combinator::choice, PResult, Parser};

/// TODO: docs
#[derive(Debug, PartialEq, Eq)]
pub enum Error<'a> {
	/// The parser unexpectedly reached the end of the input.
	End {
		/// A zero-width slice which points to the end of the input
		/// string.
		ptr: &'a str,
	},
	/// TODO: replace with concrete errors with locations.
	Unit,
}

use core::fmt;

impl fmt::Display for Error<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Error::End { .. } => {
				f.write_str("Unexpected end of input")?;
			}
			Error::Unit => f.write_str("TODO")?,
		}

		Ok(())
	}
}

impl core::error::Error for Error<'_> {}

impl Error<'_> {
	/// Creates a new `End` error which points to the end of `input`.
	pub fn end(input: &str) -> Error {
		let ptr = &input[input.len()..input.len()];
		Error::End { ptr }
	}
}

/// Returns the prefix which the inner parser consumed as output.
pub fn consume<'a, O, E>(
	parser: impl Parser<'a, &'a str, O, E>,
) -> impl Parser<'a, &'a str, &'a str, E> {
	move |input: &'a str| {
		let (_, rest) = parser.parse(input)?;

		let start = input.as_ptr() as usize;
		let end = rest.as_ptr() as usize;

		assert!(start < end);
		let length = end - start;
		assert!(length <= input.len());

		Ok((&input[..length], &input[length..]))
	}
}

impl<'a> Parser<'a, &'a str, &'a str, Error<'a>> for &str {
	fn parse(
		&self,
		input: &'a str,
	) -> PResult<&'a str, &'a str, Error<'a>> {
		if input.starts_with(self) {
			let length = self.len();
			Ok((&input[..length], &input[length..]))
		} else if input.len() < self.len() {
			Err(Error::end(input))
		} else {
			Err(Error::Unit)
		}
	}
}

impl<'a> Parser<'a, &'a str, &'a str, Error<'a>> for char {
	fn parse(
		&self,
		input: &'a str,
	) -> PResult<&'a str, &'a str, Error<'a>> {
		let needle = *self;
		char(move |ch| ch == needle).parse(input)
	}
}

/// Matches a `literal` ignoring the case.  This function is ASCII-only, all
/// other Unicode characters won't be accounted for.
///
/// ```rust
/// use komb::{Parser, string::anycase};
///
/// let p = anycase("select");
///
/// assert_eq!(Ok(("select", " from table")), p.parse("select from table"));
/// assert_eq!(Ok(("SELECT", " FROM table")), p.parse("SELECT FROM table"));
///
/// let p = anycase("löve2d");
/// assert!(p.parse("LÖVE2D").is_err());
/// ```
pub fn anycase<'a>(
	literal: &'static str,
) -> impl Parser<'a, &'a str, &'a str, Error<'a>> {
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
				return Err(Error::end(input));
			};

			if lit_ch.to_ascii_lowercase()
				!= input_ch.to_ascii_lowercase()
			{
				return Err(Error::Unit);
			}
		}
	}
}

/// Matches either a `\n` or `\r\n` line ending, returns it as an `&str`
/// reference.
///
/// ```rust
/// use komb::{Parser, string::{line_end, alphanumeric0}};
///
/// let p = alphanumeric0.before(line_end);
///
/// assert_eq!(Ok(("Hello", "world")), p.parse("Hello\nworld"));
/// ```
pub fn line_end(input: &str) -> PResult<&str, &str, Error> {
	choice(("\n", "\r\n")).parse(input)
}

/// Matches a single `\n`-terminated line.
///
/// Returns the whole line excluding the terminating newline character.  If the
/// line ended in `\r\n`, carriage return will be part of the output.
///
/// ```rust
/// use komb::{Parser, string::line};
///
/// assert_eq!(Ok(("Hello", "world")), line.parse("Hello\nworld"));
/// assert_eq!(Ok(("Hello\r", "world")), line.parse("Hello\r\nworld"));
/// assert_eq!(Ok(("", "next line")), line.parse("\nnext line"));
/// assert!(line.parse("").is_err());
/// // No newline at the end
/// assert!(line.parse("Hello there").is_err());
/// ```
pub fn line(input: &str) -> PResult<&str, &str, Error> {
	none_of0(&['\n']).before(line_end).parse(input)
}

/// Succeeds if the input is empty.
///
/// ```rust
/// use komb::{Parser, string::eof};
///
/// let p = "Hello world".before(eof);
///
/// assert_eq!(Ok(("Hello world", "")), p.parse("Hello world"));
/// assert!(p.parse("Hello world and then some").is_err());
/// ```
pub fn eof(input: &str) -> PResult<&str, (), Error> {
	if input.is_empty() {
		Ok(((), input))
	} else {
		Err(Error::Unit)
	}
}

/// Takes exactly `length` characters (not bytes) from the input.  Returns
/// [`Error::End`] if the string isn't long enough.
pub fn take<'a>(length: usize) -> impl Parser<'a, &'a str, &'a str, Error<'a>> {
	move |input: &'a str| {
		let mut current_length = 0;
		for (i, ch) in input.char_indices() {
			current_length += 1;
			if current_length == length {
				let split = i + ch.len_utf8();
				return Ok((&input[..split], &input[split..]));
			}
		}

		Err(Error::end(input))
	}
}

macro_rules! doc0to1 {
	() => {
		"Note that this will succeed even if it matches no characters. In this case it'll return an empty string pointing to the start of input as the output.  To fail in such cases use "
	};
}

macro_rules! doc1to0 {
	() => {
		"Note that this will fail if it can't match at least a single character.  To return an empty string in such cases use "
	};
}

/// Cuts off a prefix of a string for whose characters the predicate `f` returns
/// `true`.
///
/// If it hits the end of the string, the whole string will be returned and the
/// remainder will be an empty string pointing to the end of input.
///
///
/// ```rust
/// use komb::{Parser, string::take_while0};
///
/// let p = take_while0(|ch| ch.is_alphanumeric());
/// assert_eq!(Ok(("abc1", " and rest")), p.parse("abc1 and rest"));
/// assert_eq!(Ok(("", "-_-")), p.parse("-_-"))
/// ```
///
#[doc=concat!(doc0to1!(), "[`", "take_while1", "`]")]
pub fn take_while0<'a, F>(f: F) -> impl Parser<'a, &'a str, &'a str, Error<'a>>
where
	F: Fn(char) -> bool + 'a,
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

/// Matches a prefix until the first character which satisfies the predicate.
///
#[doc=concat!(doc0to1!(), "[`", "take_until1", "`]")]
pub fn take_until0<'a, F>(f: F) -> impl Parser<'a, &'a str, &'a str, Error<'a>>
where
	F: Fn(char) -> bool + 'a,
{
	take_while0(move |c| !f(c))
}

/// Matches the characters in `chars`.
///
#[doc=concat!(doc0to1!(), "[`", "one_of1", "`]")]
pub fn one_of0<'a, 'c: 'a>(
	chars: &'c [char],
) -> impl Parser<'a, &'a str, &'a str, Error<'a>> {
	take_while0(move |c| chars.contains(&c))
}

/// Matches the characters not in `chars`.
///
#[doc=concat!(doc0to1!(), "[`", "none_of1", "`]")]
pub fn none_of0<'a, 'c: 'a>(
	chars: &'c [char],
) -> impl Parser<'a, &'a str, &'a str, Error<'a>> {
	take_until0(move |c| chars.contains(&c))
}

/// Matches Unicode whitespace.
///
/// Uses [`char::is_whitespace`].
///
#[doc=concat!(doc0to1!(), "[`", "whitespace1", "`]")]
pub fn whitespace0(input: &str) -> PResult<&str, &str, Error> {
	take_while0(|c| c.is_whitespace()).parse(input)
}

/// Matches alphabetic characters.
///
/// Uses [`char::is_alphabetic`].
///
#[doc=concat!(doc0to1!(), "[`", "alphabetic0", "`]")]
pub fn alphabetic0(input: &str) -> PResult<&str, &str, Error> {
	take_while0(|c| c.is_alphabetic()).parse(input)
}

/// Matches alphanumeric characters.
///
/// Uses [`char::is_alphanumeric`].
///
/// ```rust
/// use komb::{Parser, string::alphanumeric0};
///
/// let p = alphanumeric0;
///
/// assert_eq!(Ok(("abc0", " rest")), p.parse("abc0 rest"));
/// assert_eq!(Ok(("", "-_-")), p.parse("-_-"));
/// assert_eq!(Ok(("", "")), p.parse(""));
/// ```
///
#[doc=concat!(doc0to1!(), "[`", "alphanumeric1", "`]")]
pub fn alphanumeric0(input: &str) -> PResult<&str, &str, Error> {
	take_while0(|c| c.is_alphanumeric()).parse(input)
}

/// Cuts off a prefix of a string for whose characters the predicate `f` returns
/// `true`.
///
/// ```rust
/// use komb::{Parser, string::take_while1};
///
/// let p = take_while1(|ch| ch == '0' || ch == '1');
///
/// assert_eq!(Ok(("01010", "rest")), p.parse("01010rest"));
/// assert!(p.parse("other").is_err());
/// ```
///
#[doc=concat!(doc1to0!(), "[`", "take_while0", "`]")]
pub fn take_while1<'a, F>(f: F) -> impl Parser<'a, &'a str, &'a str, Error<'a>>
where
	F: Fn(char) -> bool + 'a,
{
	move |input: &'a str| {
		let mut index = 0;
		let mut at_least_one = false;

		for (i, char) in input.char_indices() {
			if !f(char) {
				return if at_least_one {
					Ok((&input[..i], &input[i..]))
				} else {
					Err(Error::end(input))
				};
			}
			index = i + char.len_utf8();
			at_least_one = true;
		}

		if at_least_one {
			Ok((&input[..index], &input[index..]))
		} else {
			Err(Error::end(input))
		}
	}
}

/// Matches a prefix until the first character which satisfies the predicate.
///
#[doc=concat!(doc1to0!(), "[`", "take_until0", "`]")]
pub fn take_until1<'a, F>(f: F) -> impl Parser<'a, &'a str, &'a str, Error<'a>>
where
	F: Fn(char) -> bool + 'a,
{
	take_while1(move |c| !f(c))
}

/// Matches the characters in `chars`.
///
#[doc=concat!(doc1to0!(), "[`", "one_of0", "`]")]
pub fn one_of1<'a, 'c: 'a>(
	chars: &'c [char],
) -> impl Parser<'a, &'a str, &'a str, Error<'a>> {
	take_while1(move |c| chars.contains(&c))
}

/// Matches the characters not in `chars`.
///
#[doc=concat!(doc1to0!(), "[`", "none_of0", "`]")]
pub fn none_of1<'a, 'c: 'a>(
	chars: &'c [char],
) -> impl Parser<'a, &'a str, &'a str, Error<'a>> {
	take_until1(move |c| chars.contains(&c))
}

/// Matches Unicode whitespace.
///
/// Uses [`char::is_whitespace`].
///
#[doc=concat!(doc1to0!(), "[`", "whitespace0", "`]")]
pub fn whitespace1(input: &str) -> PResult<&str, &str, Error> {
	take_while1(|c| c.is_whitespace()).parse(input)
}

/// Matches alphabetic characters.
///
/// Uses [`char::is_alphabetic`].
///
#[doc=concat!(doc1to0!(), "[`", "alphabetic1", "`]")]
pub fn alphanumeric1(input: &str) -> PResult<&str, &str, Error> {
	take_while1(|c| c.is_alphanumeric()).parse(input)
}

/// Matches alphanumeric characters.
///
/// Uses [`char::is_alphanumeric`].
///
/// ```rust
/// use komb::{Parser, string::alphabetic1};
///
/// let p = alphabetic1;
///
/// assert_eq!(Ok(("abcXYZ", " rest")), p.parse("abcXYZ rest"));
/// assert!(p.parse("_ident").is_err());
/// ```
///
#[doc=concat!(doc1to0!(), "[`", "alphanumeric1", "`]")]
pub fn alphabetic1(input: &str) -> PResult<&str, &str, Error> {
	take_while1(|c| c.is_alphabetic()).parse(input)
}

// Character combinators

/// Returns the first character in input if it satisfies the predicate.
///
/// If the predicate fails, [`Error::Unit`] is returned.  If the string is
/// empty, [`Error::End`] is returned.
///
/// This function returns a borrowed string slice `&str` to preserve the
/// location of the character.
///
/// ```rust
/// use komb::{Parser, string::char};
///
/// let p = char(|ch| ch == '1' || ch == 'a');
///
/// assert_eq!(Ok(("1", "rest")), p.parse("1rest"));
/// assert_eq!(Ok(("a", "1")), p.parse("a1"));
/// assert!(p.parse("x").is_err());
/// ```
pub fn char<'a, F>(f: F) -> impl Parser<'a, &'a str, &'a str, Error<'a>>
where
	F: Fn(char) -> bool + 'a,
{
	move |input: &'a str| {
		let Some(ch) = input.chars().next() else {
			return Err(Error::end(input));
		};

		if f(ch) {
			let lenght = ch.len_utf8();
			Ok((&input[..lenght], &input[lenght..]))
		} else {
			Err(Error::Unit)
		}
	}
}

/// Returns whatever char is first in input.  It can return [`Error::End`]
/// if the input is empty.
pub fn any_char(input: &str) -> PResult<&str, &str, Error> {
	char(|_| true).parse(input)
}

/// Returns the first input char if it's one of `chars`.
pub fn one_of_char<'a, 'c: 'a>(
	chars: &'c [char],
) -> impl Parser<'a, &'a str, &'a str, Error<'a>> {
	char(|ch| chars.contains(&ch))
}

/// Returns the first input char if it's *not* one of `chars`.
pub fn none_of_char<'a, 'c: 'a>(
	chars: &'c [char],
) -> impl Parser<'a, &'a str, &'a str, Error<'a>> {
	char(|ch| !chars.contains(&ch))
}

/// Matches digits in a radix.
///
/// The behaviour is the as that of [`char::is_digit`].
///
/// ```rust
/// use komb::{Parser, string::digits1};
///
/// let p = digits1(16);
///
/// assert_eq!(Ok(("deadbeef", "rest")), p.parse("deadbeefrest"));
/// ```
pub fn digits1<'a>(radix: u32) -> impl Parser<'a, &'a str, &'a str, Error<'a>> {
	take_while1(move |c| c.is_digit(radix))
}

macro_rules! impl_parse_uint {
	($name:ident, $type:ty) => {
		#[doc=concat!("Parses a decimal ", stringify!($type), ".")]
		///
		/// Plus or minus signs aren't accepted.
		pub fn $name<'a>(
		) -> impl Parser<'a, &'a str, ($type, &'a str), Error<'a>> {
			|input: &'a str| {
				let (s, rest) = digits1(10)
					.parse(input)
					.map_err(|_| Error::Unit)?;
				let out = s.parse().map_err(|_| Error::Unit)?;

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
