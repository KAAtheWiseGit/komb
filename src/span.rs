use core::ops::Range;

/// A span denoting a subslice of a string or a byte slice, equivalent to a
/// half-open range.  Unlike the latter, it guarantees that start is always
/// smaller than the end.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Span {
	start: usize,
	end: usize,
}

impl Span {
	/// Creates a new span from `start` to `end`.
	///
	/// # Panics
	///
	/// Panics if start isn't less or equal to end.  See [`Span::try_new`]
	/// for an infallible constructor.
	pub fn new(start: usize, end: usize) -> Self {
		assert!(start <= end, "start must be less or equal to end");
		Span { start, end }
	}

	/// Returns a new span or `None` if `start` isn't less or equal to
	/// `end`.
	pub fn try_new(start: usize, end: usize) -> Option<Self> {
		if start <= end {
			Some(Span { start, end })
		} else {
			None
		}
	}

	/// Determines a span of `needle` inside the `haystack`.  If `needle`
	/// isn't a substring of the `haystack`, returns `None`.  It is
	/// guaranteed that, in absence of mutations, `haystack[span.to_range()]
	/// == needle`.
	pub fn from_substring(needle: &str, haystack: &str) -> Option<Self> {
		// See <https://doc.rust-lang.org/stable/std/primitive.slice.html#method.subslice_range>
		let (needle, haystack) =
			(needle.as_bytes(), haystack.as_bytes());

		let needle_start = needle.as_ptr() as usize;
		let haystack_start = haystack.as_ptr() as usize;

		let start = needle_start.wrapping_sub(haystack_start);
		let end = start.wrapping_add(needle.len());

		if start <= haystack.len() && end <= haystack.len() {
			Some(Self::new(start, end))
		} else {
			None
		}
	}

	/// An immutable getter.
	pub fn start(&self) -> usize {
		self.start
	}

	/// An immutable getter.
	pub fn end(&self) -> usize {
		self.end
	}

	/// Converts the span to an equivalent range, which can be used to index
	/// strings and byte slices.
	pub fn to_range(&self) -> Range<usize> {
		self.start..self.end
	}

	/// Returns `true` if `self` fully contains `other`.
	pub fn contains(&self, other: Span) -> bool {
		self.start <= other.start && self.end >= other.end
	}

	/// Length of the slice.  Note that this method returns the length in
	/// bytes, not Unicode code points or characters.
	pub fn len(&self) -> usize {
		self.end - self.start
	}

	/// Returns `true` if the span doesn't contain a single value.
	pub fn is_empty(&self) -> bool {
		self.start == self.end
	}
}

impl From<Range<usize>> for Span {
	fn from(value: Range<usize>) -> Self {
		Span {
			start: value.start,
			end: value.end,
		}
	}
}

impl From<Span> for Range<usize> {
	fn from(value: Span) -> Self {
		value.to_range()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn from_substring() {
		let s = "an example string";
		let sub = &s[3..10];
		assert_eq!(sub, "example");

		let span = Span::from_substring(sub, s);
		assert_eq!(Some(Span::new(3, 10)), span);
		let span = span.unwrap();
		assert_eq!("example", &s[span.to_range()]);

		let other = "another string";
		assert_eq!(None, Span::from_substring(other, s));
	}
}
