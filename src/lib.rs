// TODO: examples
#![doc = include_str!("../README.md")]
#![no_std]
#![allow(missing_docs)]
#![allow(dead_code)]
extern crate alloc;

pub mod combinator;
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
pub trait Parser<'a, I, O, E> {
	/// The core parsing method.
	///
	/// See the general [`Parser`] and [`PResult`] documentations for
	/// pointers on implementing it.
	fn parse(&self, input: I) -> PResult<I, O, E>;

	/// Creates a copy of the parser.
	fn clone(&self) -> impl Parser<I, O, E>
	where
		Self: Sized,
	{
		move |input| self.parse(input)
	}

	/// Converts the output type using the `Into` trait.
	fn coerce<'s, OX>(self) -> impl Parser<'s, I, OX, E>
	where
		Self: Sized + 's,
		O: Into<OX>,
	{
		move |input| match self.parse(input) {
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
	fn map<'s, OX, F>(self, f: F) -> impl Parser<'s, I, OX, E>
	where
		Self: Sized + 's,
		I: Copy,
		F: Fn(Result<O, E>) -> Result<OX, E> + 's,
	{
		move |input| {
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
	fn map_out<'s, OX, F>(self, f: F) -> impl Parser<'s, I, OX, E>
	where
		Self: Sized + 's,
		F: Fn(O) -> OX + 's,
	{
		move |input| self.parse(input).map(|(out, rest)| (f(out), rest))
	}

	/// Applies a transformation to the error or does nothing if the parse
	/// succeeds.
	fn map_err<'s, F>(self, f: F) -> impl Parser<'s, I, O, E>
	where
		Self: Sized + 's,
		F: Fn(E) -> E + 's,
	{
		move |input| self.parse(input).map_err(&f)
	}

	/// Replace the output of a parser with `value`.
	///
	/// If the parser fails, the error remains unchanged.
	fn value<'s, OX>(self, value: OX) -> impl Parser<'s, I, OX, E>
	where
		Self: Sized + 's,
		OX: Clone + 's,
	{
		self.map_out(move |_| value.clone())
	}

	/// Calls the `other` parser if this one fails and returns it's result
	/// instead.
	fn or<'s>(
		self,
		other: impl Parser<'s, I, O, E>,
	) -> impl Parser<'s, I, O, E>
	where
		Self: Sized + 's,
		I: Copy,
	{
		move |input| self.parse(input).or_else(|_| other.parse(input))
	}

	/// Replaces the error with `default` and untouched input if the parser
	/// fails.  Similar to [`Result::or`], which it uses under the hood.
	fn or_value<'s>(self, default: O) -> impl Parser<'s, I, O, E>
	where
		Self: Sized + 's,
		I: Copy,
		O: Clone,
	{
		move |input| self.parse(input).or(Ok((default.clone(), input)))
	}

	/// If the parser succeeds, `and_then` discards the output and returns
	/// the result of the `next` parser.  If either parser fails, the error
	/// is returned immediately.
	fn and_then<'s, OX>(
		self,
		next: impl Parser<'s, I, OX, E>,
	) -> impl Parser<'s, I, OX, E>
	where
		Self: Sized + 's,
		OX: 's,
	{
		move |input| {
			self.parse(input).and_then(|(_, rest)| next.parse(rest))
		}
	}

	/// Parse `next` after `self` and discard its output.  If either parser
	/// fails, the error is returned immediately.
	fn before<'s, OX>(
		self,
		next: impl Parser<'s, I, OX, E>,
	) -> impl Parser<'s, I, O, E>
	where
		Self: Sized + 's,
		OX: 's,
	{
		move |input| {
			let (output, rest) = self.parse(input)?;
			let (_, rest) = next.parse(rest)?;

			Ok((output, rest))
		}
	}
}

impl<I, O, E, F> Parser<'_, I, O, E> for F
where
	F: Fn(I) -> Result<(O, I), E>,
{
	fn parse(&self, input: I) -> PResult<I, O, E> {
		self(input)
	}
}
