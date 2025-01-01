// TODO: examples
#![doc = include_str!("../README.md")]
#![no_std]
#![allow(missing_docs)]
#![allow(dead_code)]
extern crate alloc;

pub mod combinator;
mod context;
mod error;
pub mod string;

/// The result type returned by parsers.
///
/// The lifetime `'a` is applied to the input of type `I`, which is typically
/// unsized (usually `str` or `[u8]`).  If the parser succeeds, an `(output,
/// rest)` tuple is returned, where `output` is the type created by the parser
/// and `rest` is a sub-slice of input with the parsed part cut off.
pub type PResult<I, O, E> = Result<(O, I), E>;

/// The core trait which defines parsers.
///
/// This trait is automatically [implemented for functions][impl] which take a
/// reference to the input and return [`PResult`].  So, one can define a parsing
/// function to be used directly:
///
/// ```rust
/// use komb::{Parse, PResult, Context};
///
/// fn binary_number(input: &str) -> PResult<&str, usize> {
///     let Some(rest) = input.strip_prefix("0b") else {
///         return Err(Context::from_message("TODO").into());
///     };
///
///     // The end of digits in the number or the whole string, if it only
///     // contains digits.
///     let end = rest.find(|ch: char| !ch.is_ascii_digit()).unwrap_or(rest.len());
///
///     let (digits, rest) = (&rest[..end], &rest[end..]);
///     let Ok(out) = usize::from_str_radix(digits, 2) else {
///         return Err(Context::from_message("TODO").into());
///     };
///
///     Ok((out, rest))
/// }
///
/// assert_eq!(Ok((10, "")), binary_number.parse("0b1010"));
/// assert_eq!(Ok((33, " + 1")), binary_number.parse("0b100001 + 1"));
/// assert!(binary_number.parse("0xDEADBEEF").is_err());
/// assert!(binary_number.parse("0b012").is_err());
/// ```
///
/// Most custom parsers can be created using the provided ones.  Since these
/// return unnamed closures, `-> impl Parse<'a, I, O, E>` has to be used as the
/// return type.
///
/// ```rust
/// use komb::{Context, Parse};
///
/// fn number<'a>(radix: u32) -> impl Parse<'a, &'a str, usize> {
///     assert!(radix >= 2 && radix <= 36);
///
///     // `move` captures `radix` for each created parser
///     move |input: &'a str| {
///         let end = input
///             .find(|ch: char| !ch.is_digit(radix))
///             .unwrap_or(input.len());
///
///         let (digits, rest) = (&input[..end], &input[end..]);
///         let Ok(out) = usize::from_str_radix(digits, radix) else {
///             return Err(Context::from_message("TODO").into());
///         };
///
///         Ok((out, rest))
///     }
/// }
///
/// assert_eq!(Ok((10, "")), number(2).parse("1010"));
/// assert_eq!(Ok((7911, " + 1")), number(16).parse("1ee7 + 1"));
/// assert!(number(10).parse("Not a number").is_err());
/// ```
///
/// Finally, the `Parser` trait can be implemented manually to allow for more
/// complex behavior.  In this case only the `parse` method needs to be
/// implemented.  The implementation mustn't overwrite any of the [provided
/// methods](#provided-methods).
///
/// [impl]: #impl-Parser<'a,+I,+O,+E>-for-F
pub trait Parse<'a, I, O, E> {
	/// The core parsing method.
	///
	/// See the general [`Parser`] and [`PResult`] documentations for
	/// pointers on implementing it.
	fn parse(&self, input: I) -> PResult<I, O, E>;
}

impl<I, O, E, F> Parse<'_, I, O, E> for F
where
	F: Fn(I) -> Result<(O, I), E>,
{
	fn parse(&self, input: I) -> PResult<I, O, E> {
		self(input)
	}
}

use alloc::boxed::Box;

pub struct Parser<'a, I, O, E>(Box<dyn Parse<'a, I, O, E> + 'a>);

impl<I, O, E> Parser<'_, I, O, E> {
	pub fn parse(&self, input: I) -> PResult<I, O, E> {
		self.0.parse(input)
	}

	pub fn from<'p>(
		parser: impl Parse<'p, I, O, E> + 'p,
	) -> Parser<'p, I, O, E> {
		Parser(Box::new(parser))
	}

	/// Creates a copy of the parser.
	fn clone(&self) -> Parser<I, O, E> {
		let f = move |input| self.parse(input);
		Parser::from(f)
	}

	/// Converts the output type using the `Into` trait.
	pub fn coerce<'s, OX>(self) -> Parser<'s, I, OX, E>
	where
		Self: 's,
		O: Into<OX>,
	{
		let f = move |input| match self.parse(input) {
			Ok((out, rest)) => Ok((out.into(), rest)),
			Err(err) => Err(err),
		};
		Parser::from(f)
	}

	/// Transform the output or return an error.
	///
	/// Provides an ergonomic way to make fallible transforms on output
	/// without having to juggle the input.  The function `f` is passed
	/// `Ok(output)` if the parser succeeded or `Err(error)` if it failed.
	/// If the output gets transformed into another `Ok` output, `rest`
	/// isn't changed.  If an error is transformed into `Ok`, `rest` equals
	/// the original `input`.
	pub fn map<'s, OX, F>(self, f: F) -> Parser<'s, I, OX, E>
	where
		Self: 's,
		I: Copy,
		F: Fn(Result<O, E>) -> Result<OX, E> + 's,
	{
		let f = move |input| {
			let (res, rest) = match self.parse(input) {
				Ok((out, rest)) => {
					let res = Ok(out);
					(f(res), rest)
				}
				Err(err) => {
					let res = Err(err);
					(f(res), input)
				}
			};

			match res {
				Ok(out) => Ok((out, rest)),
				Err(err) => Err(err),
			}
		};
		Parser::from(f)
	}

	/// Applies a transformation to the output or does nothing if the parser
	/// returns an error.
	pub fn map_out<'s, OX, F>(self, f: F) -> Parser<'s, I, OX, E>
	where
		Self: 's,
		F: Fn(O) -> OX + 's,
	{
		let f = move |input| {
			self.parse(input).map(|(out, rest)| (f(out), rest))
		};
		Parser::from(f)
	}

	/// Applies a transformation to the error or does nothing if the parse
	/// succeeds.
	pub fn map_err<'s, F>(self, f: F) -> Parser<'s, I, O, E>
	where
		Self: 's,
		F: Fn(E) -> E + 's,
	{
		let f = move |input| self.parse(input).map_err(&f);
		Parser::from(f)
	}

	/// Replace the output of a parser with `value`.
	///
	/// If the parser fails, the error remains unchanged.
	pub fn value<'s, OX>(self, value: OX) -> Parser<'s, I, OX, E>
	where
		Self: 's,
		OX: Clone + 's,
	{
		self.map_out(move |_| value.clone())
	}

	/// Calls the `other` parser if this one fails and returns it's result
	/// instead.
	pub fn or<'s>(self, other: Parser<'s, I, O, E>) -> Parser<'s, I, O, E>
	where
		Self: 's,
		I: Copy,
	{
		let f = move |input| {
			self.parse(input).or_else(|_| other.parse(input))
		};
		Parser::from(f)
	}

	/// Replaces the error with `default` and untouched input if the parser
	/// fails.  Similar to [`Result::or`], which it uses under the hood.
	pub fn or_value<'s>(self, default: O) -> Parser<'s, I, O, E>
	where
		Self: 's,
		I: Copy,
		O: Clone,
	{
		let f = move |input| {
			self.parse(input).or(Ok((default.clone(), input)))
		};
		Parser::from(f)
	}

	/// If the parser succeeds, `and_then` discards the output and returns
	/// the result of the `next` parser.  If either parser fails, the error
	/// is returned immediately.
	pub fn and_then<'s, OX>(
		self,
		next: Parser<'s, I, OX, E>,
	) -> Parser<'s, I, OX, E>
	where
		Self: 's,
		OX: 's,
	{
		let f = move |input| {
			self.parse(input).and_then(|(_, rest)| next.parse(rest))
		};
		Parser::from(f)
	}

	/// Parse `next` after `self` and discard its output.  If either parser
	/// fails, the error is returned immediately.
	pub fn before<'s, OX>(
		self,
		next: Parser<'s, I, OX, E>,
	) -> Parser<'s, I, O, E>
	where
		Self: 's,
		OX: 's,
	{
		let f = move |input| {
			let (output, rest) = self.parse(input)?;
			let (_, rest) = next.parse(rest)?;

			Ok((output, rest))
		};
		Parser::from(f)
	}
}
