//! Komb is a basic parser combinator library.  It borrows the source directly
//! operating on `&str` and `[u8]`, allowing zero-copy parsing.  Due to this
//! Komb doesn't support streaming.  It is also generic over the error type and
//! doesn't provide its own.

// TODO: point to examples

#![cfg_attr(not(test), no_std)]

pub mod combinator;
mod span;
pub mod string;

pub use span::Span;

/// The result type returned by parsers.
///
/// The lifetime `'a` is applied to the input of type `I`, which is typically
/// unsized (usually `str` or `[u8]`).  If the parser succeeds, an `(output,
/// rest)` tuple is returned, where `output` is the type created by the parser
/// and `rest` is a sub-slice of input with the parsed part cut off.
pub type PResult<'a, I, O, E> = Result<(O, &'a I), E>;

/// The core trait which defines parsers.
///
/// This trait is automatically [implemented for functions][impl] which take a
/// reference to the input and return [`PResult`].  So, one can define a parsing
/// function to be used directly:
///
/// ```rust
/// use komb::{Parser, PResult};
///
/// #[derive(Debug, PartialEq)]
/// struct MyError;
///
/// fn binary_number<'a>(input: &'a str) -> PResult<'a, str, usize, MyError> {
///     let Some(rest) = input.strip_prefix("0b") else {
///         return Err(MyError);
///     };
///
///     // The end of digits in the number or the whole string, if it only
///     // contains digits.
///     let end = rest.find(|ch: char| !ch.is_ascii_digit()).unwrap_or(rest.len());
///
///     let (digits, rest) = (&rest[..end], &rest[end..]);
///     let Ok(out) = usize::from_str_radix(digits, 2) else {
///         return Err(MyError);
///     };
///
///     Ok((out, rest))
/// }
///
/// # fn main() {
/// assert_eq!(Ok((10, "")), binary_number.parse("0b1010"));
/// assert_eq!(Ok((33, " + 1")), binary_number.parse("0b100001 + 1"));
/// assert_eq!(Err(MyError), binary_number.parse("0xDEADBEEF"));
/// assert_eq!(Err(MyError), binary_number.parse("0b012"));
/// # }
/// ```
///
/// Most custom parsers can be created using the provided ones.  Since these
/// return unnamed closures, `-> impl Parser<'a, I, O, E>` has to be used as the
/// return type.
///
/// ```rust
/// use komb::Parser;
///
/// #[derive(Debug, PartialEq)]
/// struct MyError;
///
/// fn number<'a>(radix: u32) -> impl Parser<'a, str, usize, MyError> {
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
///             return Err(MyError);
///         };
///
///         Ok((out, rest))
///     }
/// }
///
/// # fn main() {
/// assert_eq!(Ok((10, "")), number(2).parse("1010"));
/// assert_eq!(Ok((7911, " + 1")), number(16).parse("1ee7 + 1"));
/// assert_eq!(Err(MyError), number(10).parse("Not a number"));
/// # }
/// ```
///
/// Finally, the `Parser` trait can be implemented manually to allow for more
/// complex behavior.  In this case only the `parse` method needs to be
/// implemented.  The implementation mustn't overwrite any of the [provided
/// methods](#provided-methods).
///
/// [impl]: #impl-Parser<'a,+I,+O,+E>-for-F
pub trait Parser<'a, I, O, E>
where
	I: 'a + ?Sized,
{
	fn parse(&self, input: &'a I) -> PResult<'a, I, O, E>;

	/// Converts output and/or error types.  Useful for wrapping errors.
	fn into<OX, EX>(self) -> impl Parser<'a, I, OX, EX>
	where
		Self: Sized,
		O: Into<OX>,
		E: Into<EX>,
	{
		move |input: &'a I| match self.parse(input) {
			Ok((out, rest)) => Ok((out.into(), rest)),
			Err(err) => Err(err.into()),
		}
	}

	/// Applies a transformation to the output or does nothing if the parser
	/// returns an error.
	fn map_out<OX, F>(self, f: F) -> impl Parser<'a, I, OX, E>
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
	fn map_err<EX, F>(self, f: F) -> impl Parser<'a, I, O, EX>
	where
		Self: Sized,
		F: Fn(E) -> EX,
	{
		move |input: &'a I| self.parse(input).map_err(&f)
	}

	/// Calls the `other` parser if this one fails and returns it's result
	/// instead.
	fn or<Q>(self, other: Q) -> impl Parser<'a, I, O, E>
	where
		Self: Sized,
		Q: Parser<'a, I, O, E>,
	{
		move |input: &'a I| {
			self.parse(input).or_else(|_| other.parse(input))
		}
	}

	/// Replaces the error with `default` and untouched input if the parser
	/// fails.  Similar to [`Result::or`], which it uses under the hood.
	fn or_value(self, default: O) -> impl Parser<'a, I, O, E>
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
	fn and_then<Q, OX>(self, next: Q) -> impl Parser<'a, I, OX, E>
	where
		Self: Sized,
		Q: Parser<'a, I, OX, E>,
	{
		move |input: &'a I| {
			self.parse(input).and_then(|(_, rest)| next.parse(rest))
		}
	}

	/// Parse `next` after `self` and discard its output.  If either parser
	/// fails, the error is returned immediately.
	fn before<Q, OX>(self, next: Q) -> impl Parser<'a, I, O, E>
	where
		Self: Sized,
		Q: Parser<'a, I, OX, E>,
	{
		move |input: &'a I| {
			let (output, rest) = self.parse(input)?;
			let (_, rest) = next.parse(rest)?;

			Ok((output, rest))
		}
	}
}

impl<'a, I, O, E, F> Parser<'a, I, O, E> for F
where
	I: 'a + ?Sized,
	F: Fn(&'a I) -> PResult<'a, I, O, E>,
{
	fn parse(&self, input: &'a I) -> PResult<'a, I, O, E> {
		self(input)
	}
}
