#![allow(unused)]

use std::collections::HashMap;
use std::convert::Infallible;

use komb::{
	combinator::{choice, delimited, fold, optional},
	string::{none_of_char, one_of0},
	PResult, Parser,
};

#[derive(Clone)]
enum Value {
	Null,
	Bool(bool),
	Number(f64),
	String(String),
	Array(Vec<Value>),
	Object(HashMap<String, Value>),
}

fn whitespace(input: &str) -> PResult<str, (), ()> {
	let out = one_of0(&[' ', '\n', '\r', '\t'])
		.map_out(|_| ())
		.map_err(|_| ())
		.parse(input);
	out
}

fn bool(input: &str) -> PResult<str, bool, ()> {
	delimited(
		whitespace,
		choice((
			"true".map_out(|_| true).map_err(|_| ()),
			"false".map_out(|_| false).map_err(|_| ()),
		)),
		whitespace,
	)
	.parse(input)
}

fn string(input: &str) -> PResult<str, String, ()> {
	let p = fold(
		choice((
			"\\\"".map_out(|_| '\"'),
			"\\\\".map_out(|_| '\\'),
			"\\/".map_out(|_| '/'),
			"\\b".map_out(|_| '\x08'),
			"\\f".map_out(|_| '\x0C'),
			"\\n".map_out(|_| '\n'),
			"\\r".map_out(|_| '\r'),
			"\\t".map_out(|_| '\t'),
			none_of_char(&['\\', '"']),
		)),
		String::new(),
		|acc, ch| acc.push(ch),
	)
	.map_err(|_| ());

	delimited(
		'"'.map_err(|_| ()),
		p,
		'"'.map_err(|e| ()),
	)
	.parse(input)
}

fn array(input: &str) -> PResult<str, Vec<Value>, ()> {
	let p = value.before(optional(',').map_err(|_| ()));
	let folded = fold(p, Vec::new(), |acc, value| acc.push(value));
	let delimited = delimited(
		'['.map_err(|_| ()),
		folded,
		']'.map_err(|_| ()),
	);

	delimited.parse(input)
}

fn value(input: &str) -> PResult<str, Value, ()> {
	todo!()
}

#[test]
fn test_bool() {
	let s = "   true  ";
	assert_eq!(Ok((true, "")), bool(s));
}

#[test]
fn test_string() {
	let s = r#""a string with \"escapes\n""#;

	assert_eq!(
		Ok((String::from("a string with \"escapes\n"), "")),
		string.parse(s)
	)
}
