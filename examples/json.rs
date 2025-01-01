#![allow(missing_docs)]
#![allow(dead_code)]

use std::collections::HashMap;

use komb::{
	combinator::{choice, delimited, fold, optional, tuple},
	string::{eof, literal, none_of_char, one_of0, take},
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
	let u_esc = literal("\\u").and_then(take(4).map(|v| {
		let s = match v {
			Ok(s) => s,
			Err(_) => return Err(()),
		};
		let num = u32::from_str_radix(s, 16).map_err(|_| ())?;
		Ok(char::from_u32(num).unwrap())
	}));

	let character = choice((
		literal("\\").value('\"'),
		literal("\\\\").value('\\'),
		literal("\\/").value('/'),
		literal("\\b").value('\x08'),
		literal("\\f").value('\x0C'),
		literal("\\n").value('\n'),
		literal("\\r").value('\r'),
		literal("\\t").value('\t'),
		u_esc,
		none_of_char(&['\\', '"']),
	));

	let p = fold(character, String::new(), |acc, ch| acc.push(ch));

	delimited(literal("\""), p, literal("\""))
		.coerce()
		.parse(input)
}

fn object(input: &str) -> PResult<&str, HashMap<String, Value>, ()> {
	let pair = tuple((
		Parser::from(whitespace),
		Parser::from(string),
		Parser::from(whitespace),
		literal(":"),
		Parser::from(value),
		optional(literal(",")),
		Parser::from(whitespace),
	))
	.map_out(|tuple| (tuple.1, tuple.4));

	let folded = fold(pair, HashMap::new(), |acc, (k, v)| {
		acc.insert(k, v);
	});

	delimited(
		literal("{").before(Parser::from(whitespace)),
		folded,
		literal("}"),
	)
	.parse(input)
}

fn array(input: &str) -> PResult<&str, Vec<Value>, ()> {
	let comma = optional(literal(","));

	let folded = fold(
		Parser::from(value).before(comma),
		Vec::new(),
		|acc, value| acc.push(value),
	);

	delimited(literal("["), folded, literal("]")).parse(input)
}

fn value(input: &str) -> PResult<&str, Value, ()> {
	delimited(
		Parser::from(whitespace),
		choice((
			Parser::from(string).map_out(Value::String),
			Parser::from(object).map_out(Value::Object),
			Parser::from(array).map_out(Value::Array),
			literal("true").value(Value::Bool(true)),
			literal("false").value(Value::Bool(false)),
			literal("null").value(Value::Null),
		)),
		Parser::from(whitespace),
	)
	.parse(input)
}

fn parser(input: &str) -> PResult<&str, Value, ()> {
	Parser::from(value).before(eof()).parse(input)
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

	let s = std::hint::black_box(s);

	for _ in 0..10_000 {
		let _ = parser(s);
	}
}
