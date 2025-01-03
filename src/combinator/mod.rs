//! Type-agnostic combinators which combine other parsers.

mod choice;
mod tuple;
pub use choice::choice;

use crate::Parser;

/// Makes the passed parser optional.  That is, it'll return `Ok((None, input))`
/// if the underlying parser fails.  The input won't be consumed.
///
/// ```rust
/// use komb::{Parser, combinator::optional};
///
/// let p = optional("lit");
///
/// assert_eq!(Ok((Some("lit"), " rest")), p.parse("lit rest"));
/// assert_eq!(Ok((None, "lat rest")), p.parse("lat rest"));
/// ```
pub fn optional<'a, I, O, E>(
	parser: impl Parser<'a, I, O, E>,
) -> impl Parser<'a, I, Option<O>, E>
where
	I: Copy + 'a,
	O: 'a,
	E: 'a,
{
	move |input| match parser.parse(input) {
		Ok((out, rest)) => Ok((Some(out), rest)),
		Err(_) => Ok((None, input)),
	}
}

/// Swaps the parser results: if the underlying parser succeeds, `not` will
/// return the output wrapped in `Err`.  If it fails, `not` parser will return
/// `Ok` with the error and the same input which was passed to it.  In case of
/// an error the input is not consumed.
///
/// ```rust
/// use komb::{Parser, combinator::not};
///
/// let p = not("str");
///
/// assert!(p.parse("str").is_err());
/// assert!(p.parse("other").is_ok());
/// assert_eq!("other", p.parse("other").unwrap().1);
/// ```
pub fn not<'a, I, O, E>(
	parser: impl Parser<'a, I, O, E>,
) -> impl Parser<'a, I, E, O>
where
	I: Copy + 'a,
	O: 'a,
	E: 'a,
{
	move |input| match parser.parse(input) {
		Ok((output, _)) => Err(output),
		Err(err) => Ok((err, input)),
	}
}

// TODO: investigate discarding the delimiting parsers errors and returning a
// custom one instead.  This will allow to mix the error types of the parsers,
// avoiding the `map_err` transforms required right now.
/// Matches the `content` parser with `left` and `right` at the start and the
/// end respectively.  If any one of the three parsers fails, this error is
/// returned.  Otherwise the input of `content` is returned.
///
/// ```rust
/// use komb::{Parser, combinator::delimited};
/// use komb::string::{alphabetic, or0};
///
/// let p = delimited("(", or0(alphabetic), ")");
///
/// assert_eq!(Ok(("word", "")), p.parse("(word)"));
/// assert_eq!(Ok(("", " rest")), p.parse("() rest"));
/// assert!(p.parse("(notclosed").is_err());
/// ```
pub fn delimited<'a, I, OL, O, OR, E>(
	left: impl Parser<'a, I, OL, E> + 'a,
	content: impl Parser<'a, I, O, E> + 'a,
	right: impl Parser<'a, I, OR, E> + 'a,
) -> impl Parser<'a, I, O, E>
where
	I: Copy + 'a,
	OL: 'a,
	O: 'a,
	OR: 'a,
	E: 'a,
{
	(left, content, right).map_out(|tuple| tuple.1)
}

/// Applies `parser` and passes its output to the `apply`, which can modify the
/// `acc` accumulator.  Useful for building strings, vectors of AST elements,
/// and so on.
///
/// Note that `fold` will stop when `parser` returns an error.  This means that
/// if `parser` never errors out, `fold` will hang forever.
///
/// ```rust
/// use komb::{Parser, combinator::fold, string::any_char};
///
/// let p = fold(
///     any_char.before(","),
///     Vec::new(),
///     |v, element| {
///         v.push(element)
///     }
/// );
///
/// let (output, _) = p.parse("a,b,c,d,").unwrap();
///
/// assert_eq!(vec!["a", "b", "c", "d"], output);
/// ```
pub fn fold<'a, I, O, OX, E, F>(
	parser: impl Parser<'a, I, O, E>,
	acc: OX,
	apply: F,
) -> impl Parser<'a, I, OX, E>
where
	I: Copy + 'a,
	O: 'a,
	OX: Clone + 'a,
	E: 'a,
	F: Fn(&mut OX, O) + 'a,
{
	move |input| {
		let mut acc = acc.clone();
		let mut input = input;

		loop {
			let Ok((output, rest)) = parser.parse(input) else {
				break;
			};
			input = rest;
			apply(&mut acc, output);
		}

		Ok((acc, input))
	}
}
