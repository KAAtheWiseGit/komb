#![allow(missing_docs)]
#![allow(dead_code)]

use std::collections::HashMap;

use komb::{
	combinator::{choice, delimited, fold, optional},
	string::{eof, none_of_char, one_of0, take, StringError},
	PResult, Parser, Span,
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

#[derive(Debug, Default, PartialEq, Eq)]
struct Error {
	message: String,
	location: Option<Span>,
}

impl Error {
	fn new<S: AsRef<str>>(msg: S) -> Error {
		Error {
			message: msg.as_ref().to_owned(),
			location: None,
		}
	}

	fn unreachable() -> Error {
		Error::new("Unreachable")
	}
}

impl From<StringError> for Error {
	fn from(val: StringError) -> Error {
		Error::new(val.to_string())
	}
}

fn whitespace<'a>() -> impl Parser<'a, str, (), Error> {
	one_of0(&[' ', '\n', '\r', '\t'])
		.map_out(|_| ())
		.map_err(|_| Error::unreachable())
}

fn string<'a>() -> impl Parser<'a, str, String, Error> {
	let u_esc =
		"\\u".map_err(|_| Error::unreachable())
			.and_then(take(4).map(|v| {
				let s = match v {
					Ok(s) => s,
					Err(_) => return Err(Error::new(
						"Invalid character code",
					)),
				};
				let num = u32::from_str_radix(s, 16).map_err(
					|_| {
						Error::new("Escape code is not a valid HEX number: {s}")
					},
				)?;
				Ok(char::from_u32(num).unwrap())
			}));

	let p = fold(
		choice((
			"\\\"".map_out(|_| '\"')
				.map_err(|_| Error::unreachable()),
			"\\\\".map_out(|_| '\\')
				.map_err(|_| Error::unreachable()),
			"\\/".map_out(|_| '/')
				.map_err(|_| Error::unreachable()),
			"\\b".map_out(|_| '\x08')
				.map_err(|_| Error::unreachable()),
			"\\f".map_out(|_| '\x0C')
				.map_err(|_| Error::unreachable()),
			"\\n".map_out(|_| '\n')
				.map_err(|_| Error::unreachable()),
			"\\r".map_out(|_| '\r')
				.map_err(|_| Error::unreachable()),
			"\\t".map_out(|_| '\t')
				.map_err(|_| Error::unreachable()),
			u_esc,
			none_of_char(&['\\', '"'])
				.map_err(|_| Error::unreachable()),
		)),
		String::new(),
		|acc, ch| acc.push(ch),
	);

	delimited(
		'"'.map_err(|_| Error::unreachable()),
		p,
		'"'.map_err(|_| Error::unreachable()),
	)
	.coerce()
}

fn object<'a>() -> impl Parser<'a, str, HashMap<String, Value>, Error> {
	let comma = optional(',').map_err(|_| Error::unreachable());

	let pair = (
		whitespace(),
		string(),
		whitespace(),
		':'.coerce::<char, Error>(),
		value,
		comma,
		whitespace(),
	)
		.map_out(|tuple| (tuple.1, tuple.4));

	let folded = fold(pair, HashMap::new(), |acc, (k, v)| {
		acc.insert(k, v);
	});

	delimited(
		'{'.coerce::<char, Error>().before(whitespace()),
		folded,
		'}'.coerce::<char, Error>(),
	)
}

fn array<'a>() -> impl Parser<'a, str, Vec<Value>, Error> {
	let comma = optional(',').map_err(|_| Error::unreachable());

	let folded = fold(value.before(comma), Vec::new(), |acc, value| {
		acc.push(value)
	});

	delimited(
		'['.coerce::<char, Error>(),
		folded,
		']'.coerce::<char, Error>(),
	)
}

fn value(input: &str) -> PResult<str, Value, Error> {
	delimited(
		whitespace(),
		choice((
			string().map_out(Value::String),
			object().map_out(Value::Object),
			array().map_out(Value::Array),
			"true".map_out(|_| Value::Bool(true)).coerce(),
			"false".map_out(|_| Value::Bool(false)).coerce(),
			"null".map_out(|_| Value::Null).coerce(),
		)),
		whitespace(),
	)
	.map_err(|_| Error::new("Failed to match a JSON value"))
	.parse(input)
}

fn parser(input: &str) -> PResult<str, Value, Error> {
	value.before(eof().map_err(|_| Error::new("Trailing characters")))
		.parse(input)
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