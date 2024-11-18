use crate::Parser;

pub fn take_while<'i, F>(f: F) -> impl Parser<&'i str, &'i str, ()>
where
	F: Fn(char) -> bool,
{
	move |input: &'i str| {
		let mut end = 0;

		for (i, char) in input.char_indices() {
			if !f(char) {
				end = i;
				break;
			}
		}

		Ok((&input[end..], &input[..end]))
	}
}

pub fn take_until<'i, F>(f: F) -> impl Parser<&'i str, &'i str, ()>
where
	F: Fn(char) -> bool,
{
	take_while(move |c| !f(c))
}

pub fn one_of<'i>(
	chars: &[char],
) -> impl Parser<&'i str, &'i str, ()> + use<'i, '_> {
	take_while(move |c| chars.contains(&c))
}

pub fn none_of<'i>(
	chars: &[char],
) -> impl Parser<&'i str, &'i str, ()> + use<'i, '_> {
	take_until(move |c| chars.contains(&c))
}

pub fn whitespace<'i>() -> impl Parser<&'i str, &'i str, ()> {
	take_while(|c| c.is_whitespace())
}

pub fn alphanumeric<'i>() -> impl Parser<&'i str, &'i str, ()> {
	take_while(|c| c.is_alphanumeric())
}

pub fn alphabetic<'i>() -> impl Parser<&'i str, &'i str, ()> {
	take_while(|c| c.is_alphabetic())
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn playground() {
		println!("{:?}", none_of(&['c'])("abcd"));
		println!("{:?}", one_of(&['i', 'b', 'd'])("abcd"));
	}
}
