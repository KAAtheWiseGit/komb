#![allow(missing_docs)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::str::FromStr;

use komb::{
	combinator::{choice, delimited, fold, optional},
	string::{anycase, consume, digits1, eof, none_of_char, one_of0, take},
	PResult, Parser,
};

#[derive(Debug, Clone)]
enum Value {
	Null,
	Bool(bool),
	Number(f64),
	String(String),
	Array(Vec<Value>),
	Object(HashMap<String, Value>),
}

fn whitespace(input: &str) -> PResult<&str, (), ()> {
	one_of0(&[' ', '\n', '\r', '\t']).value(()).parse(input)
}

fn string(input: &str) -> PResult<&str, String, ()> {
	let u_esc = "\\u".and_then(take(4).map(|v| {
		let s = match v {
			Ok(s) => s.to_ascii_lowercase(),
			Err(_) => return Err(()),
		};
		let num = u32::from_str_radix(&s, 16).map_err(|_| ())?;
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

fn number(input: &str) -> PResult<&str, f64, ()> {
	fn digits(input: &str) -> PResult<&str, &str, ()> {
		let (out, rest) = digits1(10).parse(input)?;

		// multi-character digits cannot start with a zero
		if out.starts_with('0') && out.len() > 1 {
			return Err(());
		}

		Ok((out, rest))
	}

	let integer = (optional('-'), digits);
	let fraction = optional((('.'), digits));
	let sign = choice(('+', '-', ""));
	let exponent = optional((anycase("e"), sign, digits));

	let (number, rest) = consume((integer, fraction, exponent))
		.map_out(|s| f64::from_str(s))
		.parse(input)?;

	Ok((number.map_err(|_| ())?, rest))
}

fn object(input: &str) -> PResult<&str, HashMap<String, Value>, ()> {
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

fn array(input: &str) -> PResult<&str, Vec<Value>, ()> {
	let comma = optional(",");

	let folded = fold(value.before(comma), Vec::new(), |acc, value| {
		acc.push(value)
	});

	delimited("[", folded, "]").parse(input)
}

fn value(input: &str) -> PResult<&str, Value, ()> {
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

fn parse(input: &str) -> Result<Value, ()> {
	value.before(eof).parse(input).map(|(output, _)| output)
}

fn main() {
	let s = r#"
{
    "glossary": {
        "title": true,
	"GlossDiv": {
	    "title": "S",
	    "number": 100,
	    "another": 100.1,
	    "another2": 0.1,
	    "another3": 0.1e+10,
	    "another4": 1e+10,
	    "another5": -2E-10,
	    "GlossList": {
		"GlossEntry": {
		    "ID": "SGML",
		    "SortAs": "SGML",
		    "GlossTerm": "Standard Generalized Markup Language",
		    "Acronym": "SGML",
		    "Abbrev": "ISO 8879:1986",
		    "GlossDef": {
			"para": "A meta-markup language, used to create markup languages such as DocBook.",
			"GlossSeeAlso": ["GML", "XML"]
		    },
		    "GlossSee": "markup"
		}
	    }
	}
    }
}"#;

	let s = std::hint::black_box(s);

	for _ in 0..10_000 {
		let _ = parse(s);
	}
}
