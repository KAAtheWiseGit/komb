#![allow(missing_docs)]
#![allow(dead_code)]

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

#[derive(Debug, Clone)]
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
		let s = match v {
			Ok(s) => s.to_ascii_lowercase(),
			Err(_) => return Err(Error::Unit),
		};
		let num =
			u32::from_str_radix(&s, 16).map_err(|_| Error::Unit)?;
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
			return Err(Error::Unit);
		}

		Ok((out, rest))
	}

	let integer = (optional('-'), digits_leading);
	let fraction = optional((('.'), digits));
	let sign = choice(('+', '-', ""));
	let exponent = optional((anycase("e"), sign, digits));

	let (number, rest) = consume((integer, fraction, exponent))
		.map_out(f64::from_str)
		.parse(input)?;

	Ok((number.map_err(|_| Error::Unit)?, rest))
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

fn main() {
	let _s = r#"
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

	use std::fs::File;
	use std::io::Read;
	let mut f =
		File::open("/home/kaathewise/download/canada.json").unwrap();
	let mut s = String::new();
	f.read_to_string(&mut s).unwrap();

	let s = std::hint::black_box(&s);

	for _ in 0..100 {
		let _ = parse(s).unwrap();
	}
}
