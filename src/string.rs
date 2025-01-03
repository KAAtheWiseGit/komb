//! Concrete parsers which operate on `str` input.
//!
//! All of the parsers return [`Error`] for easier compositon.

use core::num::{ParseFloatError, ParseIntError};

use crate::{combinator::choice, PResult, Parser};

/// TODO: docs
#[derive(Debug, PartialEq, Eq)]
pub enum Error<'a> {
	/// The parser unexpectedly reached the end of the input.
	End {
		/// A zero-width slice which points to the end of the input
		/// string.
		span: &'a str,
	},
	/// The parser failed to match.
	Unmatched {
		/// The input prefix which the parser encountered instead of
		/// what it expected.
		span: &'a str,
	},
	/// Returned by [`eof`] when the input isn't empty.
	NotEnd,
	/// Failed to parse an integer.
	ParseInt {
		/// The error returned by the integer `from_str` and
		/// `from_str_radix` methods.
		error: ParseIntError,
		/// The input substring which was parsed.
		span: &'a str,
	},
	/// Failed to parse a floating point number.
	ParseFloat {
		/// The error returned by `from_str`.
		error: ParseFloatError,
		/// The input substring which was parsed.
		span: &'a str,
	},
}

use core::fmt;

impl fmt::Display for Error<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Error::End { .. } => {
				f.write_str("Unexpected end of input")?;
			}
			Error::Unmatched { span: literal } => {
				f.write_fmt(format_args!(
					"Parser failed to match '{literal}'"
				))?;
			}
			Error::NotEnd => f.write_str("Input not empty")?,
			Error::ParseInt { error, span } => {
				f.write_fmt(format_args!(
					"Failed to parse integer '{span}': "
				))?;
				error.fmt(f)?;
			}
			Error::ParseFloat { error, span } => {
				f.write_fmt(format_args!(
					"Failed to parse float '{span}': "
				))?;
				error.fmt(f)?;
			}
		}

		Ok(())
	}
}

impl core::error::Error for Error<'_> {}

impl Error<'_> {
	/// Creates a new `End` error which points to the end of `input`.
	fn end(input: &str) -> Error {
		let ptr = &input[input.len()..input.len()];
		Error::End { span: ptr }
	}

	/// Creates a new `Unmatched` error with a given span.
	pub fn unmatched(span: &str) -> Error {
		Error::Unmatched { span }
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

/// Returns an empty string if the underlying parser fails.
///
/// The string will point to the start of the input.
pub fn or0<'a, E>(
	parser: impl Parser<'a, &'a str, &'a str, E>,
) -> impl Parser<'a, &'a str, &'a str, E> {
	move |input: &'a str| {
		Ok(parser.parse(input).unwrap_or((&input[..0], input)))
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
			Err(Error::unmatched(&input[..self.len()]))
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
		let mut input_chars = input.char_indices();

		loop {
			let Some(lit_ch) = literal_chars.next() else {
				return Ok((
					&input[..length],
					&input[length..],
				));
			};

			let Some((i, input_ch)) = input_chars.next() else {
				return Err(Error::end(input));
			};

			if lit_ch.to_ascii_lowercase()
				!= input_ch.to_ascii_lowercase()
			{
				return Err(Error::unmatched(
					&input[..i + input_ch.len_utf8()],
				));
			}
		}
	}
}

/// Matches either a `\n` or `\r\n` line ending, returns it as an `&str`
/// reference.
///
/// ```rust
/// use komb::{Parser, string::{line_end, alphanumeric}};
///
/// let p = alphanumeric.before(line_end);
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
	or0(none_of(&['\n'])).before(line_end).parse(input)
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
		Err(Error::NotEnd)
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

/// Cuts off a prefix of a string for whose characters the predicate `f` returns
/// `true`.
///
/// ```rust
/// use komb::{Parser, string::take_while};
///
/// let p = take_while(|ch| ch == '0' || ch == '1');
///
/// assert_eq!(Ok(("01010", "rest")), p.parse("01010rest"));
/// assert!(p.parse("other").is_err());
/// ```
pub fn take_while<'a, F>(f: F) -> impl Parser<'a, &'a str, &'a str, Error<'a>>
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
pub fn take_until<'a, F>(f: F) -> impl Parser<'a, &'a str, &'a str, Error<'a>>
where
	F: Fn(char) -> bool + 'a,
{
	take_while(move |c| !f(c))
}

/// Matches the characters in `chars`.
pub fn one_of<'a, 'c: 'a>(
	chars: &'c [char],
) -> impl Parser<'a, &'a str, &'a str, Error<'a>> {
	take_while(move |c| chars.contains(&c))
}

/// Matches the characters not in `chars`.
pub fn none_of<'a, 'c: 'a>(
	chars: &'c [char],
) -> impl Parser<'a, &'a str, &'a str, Error<'a>> {
	take_until(move |c| chars.contains(&c))
}

/// Matches Unicode whitespace.
///
/// Uses [`char::is_whitespace`].
pub fn whitespace(input: &str) -> PResult<&str, &str, Error> {
	take_while(|c| c.is_whitespace()).parse(input)
}

/// Matches alphabetic characters.
///
/// Uses [`char::is_alphabetic`].
pub fn alphanumeric(input: &str) -> PResult<&str, &str, Error> {
	take_while(|c| c.is_alphanumeric()).parse(input)
}

/// Matches alphanumeric characters.
///
/// Uses [`char::is_alphanumeric`].
///
/// ```rust
/// use komb::{Parser, string::alphabetic};
///
/// let p = alphabetic;
///
/// assert_eq!(Ok(("abcXYZ", " rest")), p.parse("abcXYZ rest"));
/// assert!(p.parse("_ident").is_err());
/// ```
pub fn alphabetic(input: &str) -> PResult<&str, &str, Error> {
	take_while(|c| c.is_alphabetic()).parse(input)
}

// Character combinators

/// Returns the first character in input if it satisfies the predicate.
///
/// If the predicate fails, [`Error::Unmatched`] is returned.  If the string is
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

		let lenght = ch.len_utf8();
		if f(ch) {
			Ok((&input[..lenght], &input[lenght..]))
		} else {
			Err(Error::unmatched(&input[..lenght]))
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
/// Uses [`char::is_digit`] underneath.
///
/// ```rust
/// use komb::{Parser, string::digits};
///
/// let p = digits::<16>;
///
/// assert_eq!(Ok(("deadbeef", "rest")), p.parse("deadbeefrest"));
/// assert!(p.parse("").is_err());
/// ```
pub fn digits<const R: u32>(input: &str) -> PResult<&str, &str, Error> {
	take_while(move |c| c.is_digit(R)).parse(input)
}

macro_rules! impl_parse_uint {
	($type:ident) => {
		#[doc=concat!("Parses a decimal [`", stringify!($type), "`][prim@", stringify!($type), "].")]
		///
		/// Plus or minus signs aren't accepted.
		pub fn $type(input: &str) -> PResult<&str, $type, Error> {
			let (s, rest) = digits::<10>.parse(input)?;
			let out = s.parse().map_err(|error| {
				Error::ParseInt { error, span: s }
			})?;

			Ok((out, rest))
		}
	};
}

impl_parse_uint!(u8);
impl_parse_uint!(u16);
impl_parse_uint!(u32);
impl_parse_uint!(u64);
impl_parse_uint!(usize);

macro_rules! impl_parse_sint {
	($type:ident) => {
		#[doc=concat!("Parses a decimal [`", stringify!($type), "`][prim@", stringify!($type), "].")]
		///
		/// ```rust
		/// use komb::Parser;
		#[doc=concat!("use komb::string::", stringify!($type), ";")]
		///
		#[doc=concat!("assert_eq!(Ok((3, \"\")), ", stringify!($type), ".parse(\"3\"));")]
		#[doc=concat!("assert_eq!(Ok((-1, \"\")), ", stringify!($type), ".parse(\"-1\"));")]
		#[doc=concat!("assert_eq!(Ok((4, \"\")), ", stringify!($type), ".parse(\"+4\"));")]
		/// ```
		pub fn $type(input: &str) -> PResult<&str, $type, Error> {
			let sign = choice(('+', '-', ""));
			let (s, rest) =
				consume((sign, digits::<10>)).parse(input)?;
			let out = s.parse().map_err(|error| {
				Error::ParseInt { error, span: s }
			})?;

			Ok((out, rest))
		}
	};
}

impl_parse_sint!(i8);
impl_parse_sint!(i16);
impl_parse_sint!(i32);
impl_parse_sint!(i64);
impl_parse_sint!(isize);

macro_rules! impl_parse_float {
	($type:ident) => {
		#[doc=concat!("Parses a [`", stringify!($type), "`][prim@", stringify!($type), "].")]
		///
		/// This function uses Rust-style grammar.  See
		/// [`from_str`][prim@f64#method.from_str] for the description.
		///
		/// ```rust
		/// use komb::Parser;
		#[doc=concat!("use komb::string::", stringify!($type), ";")]
		///
		#[doc=concat!("assert_eq!(Ok((3.14, \"\")), ", stringify!($type), ".parse(\"3.14\"));")]
		#[doc=concat!("assert_eq!(Ok((-3.14, \"\")), ", stringify!($type), ".parse(\"-3.14\"));")]
		#[doc=concat!("assert_eq!(Ok((2.5E10, \"\")), ", stringify!($type), ".parse(\"2.5E10\"));")]
		#[doc=concat!("assert_eq!(Ok((2.5E-10, \"\")), ", stringify!($type), ".parse(\"2.5E-10\"));")]
		#[doc=concat!("assert_eq!(Ok((5.0, \"\")), ", stringify!($type), ".parse(\"5.\"));")]
		#[doc=concat!("assert_eq!(Ok((0.5, \"\")), ", stringify!($type), ".parse(\".5\"));")]
		#[doc=concat!("assert_eq!(Ok((", stringify!($type), "::INFINITY, \"\")), ", stringify!($type), ".parse(\"iNf\"));")]
		#[doc=concat!("assert_eq!(Ok((", stringify!($type), "::NEG_INFINITY, \"\")), ", stringify!($type), ".parse(\"-inF\"));")]
		/// ```
		pub fn $type(input: &str) -> PResult<&str, $type, Error> {
			use crate::combinator::optional;

			fn sign(input: &str) -> PResult<&str, (), Error> {
				optional(choice(('+', '-', "")))
					.value(())
					.parse(input)
			}
			let exp = (anycase("e"), sign, digits::<10>);
			let number = (
				choice((
					(digits::<10>, '.', or0(digits::<10>))
						.value(()),
					(or0(digits::<10>), '.', digits::<10>)
						.value(()),
					digits::<10>.value(()),
				)),
				optional(exp),
			);
			let float = (
				sign,
				choice((
					anycase("inf").value(()),
					anycase("infinity").value(()),
					anycase("nan").value(()),
					number.value(()),
				)),
			);
			let s = consume(float);

			let (span, rest) = s.parse(input)?;

			use core::str::FromStr;
			let out = $type::from_str(span).map_err(|error| {
				Error::ParseFloat { error, span }
			})?;

			Ok((out, rest))
		}
	};
}

impl_parse_float!(f32);
impl_parse_float!(f64);

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn playground() {
		assert_eq!(
			("ab", "cd"),
			none_of(&['c']).parse("abcd").unwrap()
		);
		assert_eq!(
			("ab", "cd"),
			one_of(&['a', 'b']).parse("abcd").unwrap()
		);
	}
}
