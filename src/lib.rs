// TODO: examples
#![doc = include_str!("../README.md")]
#![no_std]
extern crate alloc;

pub mod combinator;
mod context;
mod error;
pub mod string;

pub use context::Context;
pub use error::Error;

/// The result type returned by parsers.
///
/// The lifetime `'a` is applied to the input of type `I`, which is typically
/// unsized (usually `str` or `[u8]`).  If the parser succeeds, an `(output,
/// rest)` tuple is returned, where `output` is the type created by the parser
/// and `rest` is a sub-slice of input with the parsed part cut off.
pub type PResult<'a, I, O> = Result<(O, &'a I), Error>;

/// The core trait which defines parsers.
///
/// This trait is automatically [implemented for functions][impl] which take a
/// reference to the input and return [`PResult`].  So, one can define a parsing
/// function to be used directly:
///
/// ```rust
/// use komb::{Parser, PResult, Context};
///
/// fn binary_number<'a>(input: &'a str) -> PResult<'a, str, usize> {
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
/// return unnamed closures, `-> impl Parser<'a, I, O>` has to be used as the
/// return type.
///
/// ```rust
/// use komb::{Context, Parser};
///
/// fn number<'a>(radix: u32) -> impl Parser<'a, str, usize> {
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
pub trait Parser<'a, I, O>
where
	I: 'a + ?Sized,
{
	/// The core parsing method.
	///
	/// See the general [`Parser`] and [`PResult`] documentations for
	/// pointers on implementing it.
	fn parse(&self, input: &'a I) -> PResult<'a, I, O>;

	/// Converts the output type using the `Into` trait.
	fn coerce<OX>(self) -> impl Parser<'a, I, OX>
	where
		Self: Sized,
		O: Into<OX>,
	{
		move |input: &'a I| match self.parse(input) {
			Ok((out, rest)) => Ok((out.into(), rest)),
			Err(err) => Err(err),
		}
	}

	/// Transform the output or return an error.
	///
	/// Provides an ergonomic way to make fallible transforms on output
	/// without having to juggle the input.  The function `f` is passed
	/// `Ok(output)` if the parser succeeded or `Err(error)` if it failed.
	/// If the output gets transformed into another `Ok` output, `rest`
	/// isn't changed.  If an error is transformed into `Ok`, `rest` equals
	/// the original `input`.
	fn map<OX, F>(self, f: F) -> impl Parser<'a, I, OX>
	where
		Self: Sized,
		F: Fn(Result<O, Error>) -> Result<OX, Error>,
	{
		move |input: &'a I| {
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
		}
	}

	/// Applies a transformation to the output or does nothing if the parser
	/// returns an error.
	fn map_out<OX, F>(self, f: F) -> impl Parser<'a, I, OX>
	where
		Self: Sized,
		F: Fn(O) -> OX,
	{
		move |input: &'a I| {
			self.parse(input).map(|(out, rest)| (f(out), rest))
		}
	}

	/// Applies a transformation to the error or does nothing if the parse
	/// succeeds.
	fn map_err<F>(self, f: F) -> impl Parser<'a, I, O>
	where
		Self: Sized,
		F: Fn(Error) -> Error,
	{
		move |input: &'a I| self.parse(input).map_err(&f)
	}

	/// Calls the `other` parser if this one fails and returns it's result
	/// instead.
	fn or<Q>(self, other: Q) -> impl Parser<'a, I, O>
	where
		Self: Sized,
		Q: Parser<'a, I, O>,
	{
		move |input: &'a I| {
			self.parse(input).or_else(|_| other.parse(input))
		}
	}

	/// Replaces the error with `default` and untouched input if the parser
	/// fails.  Similar to [`Result::or`], which it uses under the hood.
	fn or_value(self, default: O) -> impl Parser<'a, I, O>
	where
		Self: Sized,
		O: Clone,
	{
		move |input: &'a I| {
			self.parse(input).or(Ok((default.clone(), input)))
		}
	}

	/// If the parser succeeds, `and_then` discards the output and returns
	/// the result of the `next` parser.  If either parser fails, the error
	/// is returned immediately.
	fn and_then<Q, OX>(self, next: Q) -> impl Parser<'a, I, OX>
	where
		Self: Sized,
		Q: Parser<'a, I, OX>,
	{
		move |input: &'a I| {
			self.parse(input).and_then(|(_, rest)| next.parse(rest))
		}
	}

	/// Parse `next` after `self` and discard its output.  If either parser
	/// fails, the error is returned immediately.
	fn before<Q, OX>(self, next: Q) -> impl Parser<'a, I, O>
	where
		Self: Sized,
		Q: Parser<'a, I, OX>,
	{
		move |input: &'a I| {
			let (output, rest) = self.parse(input)?;
			let (_, rest) = next.parse(rest)?;

			Ok((output, rest))
		}
	}
}

impl<'a, I, O, F> Parser<'a, I, O> for F
where
	I: 'a + ?Sized,
	F: Fn(&'a I) -> PResult<'a, I, O>,
{
	fn parse(&self, input: &'a I) -> PResult<'a, I, O> {
		self(input)
	}
}
