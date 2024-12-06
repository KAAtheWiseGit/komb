#[macro_export]
#[doc(hidden)]
macro_rules! rec_select {
	($input:ident; $p:expr) => {
		match $p.parse($input) {
			Ok((out, rest)) => Ok((out, rest)),
			Err(err) => Err(err),
		}
	};
	($input:ident; $p:expr, $($rest:expr),+ $(,)?) => {{
		if let Ok((out, rest)) = $p.parse($input) {
			return Ok((out, rest));
		};
		rec_select!($input; $($rest),+)
	}};
}

/// Applies a comma-separated sequence parsers in order returning the first
/// success or the last error, if none of the parsers match.  Note that all of
/// the parsers must have the same input, output, and error types.
///
/// It's a macro, so if there's a gigantic error trace during compilation, it
/// probably means that one of the arguments doesn't implement the
/// [`Parser`][crate::Parser] trait.
#[macro_export]
macro_rules! select {
	($($rest:expr),* $(,)?) => {
		move |input| {
			rec_select!(input; $($rest,)*)
		}
	};
}

#[cfg(test)]
mod test {
	use crate::string::char;
	use crate::Parser;

	#[test]
	fn test_macro() {
		let parser = select![char('a'), char('b'), char('c')];
		let result = parser.parse("bc");
		assert_eq!(Ok(('b', "c")), result);
	}
}
