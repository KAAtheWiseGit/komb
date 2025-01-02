#![allow(missing_docs)]

use std::collections::HashMap;
use std::str::FromStr;

use komb::{
	combinator::{choice, delimited, fold, optional},
	string::{
		anycase, consume, digits1, eof, none_of_char, one_of0, take,
		Error,
	},
	PResult, Parser,
};

#[derive(Debug, Clone, PartialEq)]
enum Value {
	Null,
	Bool(bool),
	Number(f64),
	String(String),
	Array(Vec<Value>),
	Object(HashMap<String, Value>),
}

fn whitespace(input: &str) -> PResult<&str, (), Error> {
	one_of0(&[' ', '\n', '\r', '\t']).value(()).parse(input)
}

fn string(input: &str) -> PResult<&str, String, Error> {
	let u_esc = "\\u".and_then(take(4).map(|v| {
		let s = v?;
		let num = u32::from_str_radix(s, 16)
			.map_err(|error| Error::ParseInt { error, span: s })?;
		Ok(char::from_u32(num).unwrap())
	}));

	let character = choice((
		"\\".value('\"'),
		"\\\\".value('\\'),
		"\\/".value('/'),
		"\\b".value('\x08'),
		"\\f".value('\x0C'),
		"\\n".value('\n'),
		"\\r".value('\r'),
		"\\t".value('\t'),
		u_esc,
		none_of_char(&['\\', '"'])
			.map_out(|s| s.chars().next().unwrap()),
	));

	let p = fold(character, String::new(), |acc, ch| acc.push(ch));

	delimited("\"", p, "\"").coerce().parse(input)
}

fn number(input: &str) -> PResult<&str, f64, Error> {
	fn digits(input: &str) -> PResult<&str, &str, Error> {
		let (out, rest) = digits1(10).parse(input)?;

		Ok((out, rest))
	}

	fn digits_leading(input: &str) -> PResult<&str, &str, Error> {
		let (out, rest) = digits.parse(input)?;

		// multi-character digits cannot start with a zero
		if out.starts_with('0') && out.len() > 1 {
			return Err(Error::unmatched(out));
		}

		Ok((out, rest))
	}

	let integer = (optional('-'), digits_leading);
	let fraction = optional((('.'), digits));
	let sign = choice(('+', '-', ""));
	let exponent = optional((anycase("e"), sign, digits));

	let (number_s, rest) =
		consume((integer, fraction, exponent)).parse(input)?;

	let number =
		f64::from_str(number_s).map_err(|error| Error::ParseFloat {
			error,
			span: number_s,
		})?;

	Ok((number, rest))
}

fn object(input: &str) -> PResult<&str, HashMap<String, Value>, Error> {
	let pair = (
		whitespace,
		string,
		whitespace,
		":",
		value,
		optional(","),
		whitespace,
	)
		.map_out(|tuple| (tuple.1, tuple.4));

	let folded = fold(pair, HashMap::new(), |acc, (k, v)| {
		acc.insert(k, v);
	});

	delimited("{".before(whitespace), folded, "}").parse(input)
}

fn array(input: &str) -> PResult<&str, Vec<Value>, Error> {
	let folded = fold(value.before(','), Vec::new(), |acc, value| {
		acc.push(value)
	});
	let elements = (folded, value).map_out(|(mut arr, last)| {
		arr.push(last);
		arr
	});

	delimited("[", elements, "]").parse(input)
}

fn value(input: &str) -> PResult<&str, Value, Error> {
	delimited(
		whitespace,
		choice((
			object.map_out(Value::Object),
			array.map_out(Value::Array),
			string.map_out(Value::String),
			number.map_out(Value::Number),
			"true".value(Value::Bool(true)),
			"false".value(Value::Bool(false)),
			"null".value(Value::Null),
		)),
		whitespace,
	)
	.parse(input)
}

fn parse(input: &str) -> Result<Value, Error> {
	value.before(eof).parse(input).map(|(output, _)| output)
}

fn load(file: &str) -> String {
	use std::fs::File;
	use std::io::Read;

	let mut f = File::open(file)
		.expect(&format!("Failed to open the file '{file}'"));
	let mut out = String::new();
	f.read_to_string(&mut out)
		.expect("Failed to read the file contents");
	out
}

fn main() {
	use std::env::args;

	let Some(file) = args().skip(1).next() else {
		eprintln!("Pass a UTF-8 path to a JSON file");
		return;
	};
	let json = load(&file);
	let value = parse(&json);

	println!("{:#?}", value);
}

#[test]
fn test() {
	let basic = parse(r#"["hello", "world"]"#);
	assert_eq!(
		Ok(Value::Array(vec![
			Value::String("hello".to_owned()),
			Value::String("world".to_owned())
		])),
		basic
	);

	assert!(parse(&load("data/widget.json")).is_ok());
	assert!(parse(&load("data/glossary.json")).is_ok());
	assert!(parse(&load("data/webapp.json")).is_ok());
}
