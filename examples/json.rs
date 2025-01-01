#![allow(missing_docs)]
#![allow(dead_code)]

use std::collections::HashMap;

use komb::{
	combinator::{choice, delimited, fold, optional},
	string::{eof, none_of_char, one_of0, take},
	Context, PResult, Parser,
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

fn whitespace<'a>() -> impl Parser<'a, &'a str, ()> {
	one_of0(&[' ', '\n', '\r', '\t']).value(())
}

fn string<'a>() -> impl Parser<'a, &'a str, String> {
	let u_esc = "\\u".and_then(take(4).map(|v| {
		let s = match v {
			Ok(s) => s,
			Err(_) => return Err(Context::from_message(
				"The Unicode escape must be 4 characters long",
			)
			.into()),
		};
		let num = u32::from_str_radix(s, 16)?;
		Ok(char::from_u32(num).unwrap())
	}));

	let p = fold(
		choice((
			"\\\"".value('\"'),
			"\\\\".value('\\'),
			"\\/".value('/'),
			"\\b".value('\x08'),
			"\\f".value('\x0C'),
			"\\n".value('\n'),
			"\\r".value('\r'),
			"\\t".value('\t'),
			u_esc,
			none_of_char(&['\\', '"']),
		)),
		String::new(),
		|acc, ch| acc.push(ch),
	);

	delimited('"', p, '"').coerce()
}

fn object<'a>() -> impl Parser<'a, &'a str, HashMap<String, Value>> {
	let comma = optional(',');

	let pair = (
		whitespace(),
		string(),
		whitespace(),
		':',
		value,
		comma,
		whitespace(),
	)
		.map_out(|tuple| (tuple.1, tuple.4));

	let folded = fold(pair, HashMap::new(), |acc, (k, v)| {
		acc.insert(k, v);
	});

	delimited('{'.before(whitespace()), folded, '}')
}

fn array<'a>() -> impl Parser<'a, &'a str, Vec<Value>> {
	let comma = optional(',');

	let folded = fold(value.before(comma), Vec::new(), |acc, value| {
		acc.push(value)
	});

	delimited('[', folded, ']')
}

fn value(input: &str) -> PResult<&str, Value> {
	delimited(
		whitespace(),
		choice((
			string().map_out(Value::String),
			object().map_out(Value::Object),
			array().map_out(Value::Array),
			"true".value(Value::Bool(true)).coerce(),
			"false".value(Value::Bool(false)).coerce(),
			"null".value(Value::Null).coerce(),
		)),
		whitespace(),
	)
	.parse(input)
}

fn parser(input: &str) -> PResult<&str, Value> {
	value.before(eof()).parse(input)
}

fn main() {
	let s = r#"
{
    "glossary": {
        "title": true,
	"GlossDiv": {
	    "title": "S",
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

	println!("{:#?}", parser.parse(s));
}

fn fail() {
	let s = String::from("string");
	let s: &str = &s;

	println!("{:?}", (s, s).parse("string rest"));
}
