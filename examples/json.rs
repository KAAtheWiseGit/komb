#![allow(missing_docs)]
#![allow(dead_code)]

use std::collections::HashMap;

use komb::{
	combinator::{choice, delimited, fold, optional, tuple},
	string::{eof, literal, none_of_char, one_of0, take},
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

fn whitespace<'a>() -> Parser<'a, &'a str, ()> {
	one_of0(&[' ', '\n', '\r', '\t']).value(())
}

fn string<'a>() -> Parser<'a, &'a str, String> {
	let u_esc = literal("\\u").and_then(take(4).map(|v| {
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

	delimited(literal("\""), p, literal("\"")).coerce()
}

fn object<'a>() -> Parser<'a, &'a str, HashMap<String, Value>> {
	let pair = tuple((
		whitespace(),
		string(),
		whitespace(),
		literal(":"),
		Parser::from(value),
		optional(literal(",")),
		whitespace(),
	))
	.map_out(|tuple| (tuple.1, tuple.4));

	let folded = fold(pair, HashMap::new(), |acc, (k, v)| {
		acc.insert(k, v);
	});

	delimited(literal("{").before(whitespace()), folded, literal("}"))
}

fn array<'a>() -> Parser<'a, &'a str, Vec<Value>> {
	let comma = optional(literal(","));

	let folded = fold(
		Parser::from(value).before(comma),
		Vec::new(),
		|acc, value| acc.push(value),
	);

	delimited(literal("["), folded, literal("]"))
}

fn value(input: &str) -> PResult<&str, Value> {
	delimited(
		whitespace(),
		choice((
			string().map_out(Value::String),
			object().map_out(Value::Object),
			array().map_out(Value::Array),
			literal("true").value(Value::Bool(true)),
			literal("false").value(Value::Bool(false)),
			literal("null").value(Value::Null),
		)),
		whitespace(),
	)
	.parse(input)
}

fn parser<'a>() -> Parser<'a, &'a str, Value> {
	Parser::from(value).before(eof())
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

	println!("{:#?}", parser().parse(s));
}
