#![allow(unused)]

use std::collections::HashMap;
use std::convert::Infallible;

use komb::{
	combinator::{choice, delimited, fold, option},
	string::{literal_char, literal, none_of_char, one_of0},
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
			literal("true").map_out(|_| true).map_err(|_| ()),
			literal("false").map_out(|_| false).map_err(|_| ()),
		)),
		whitespace,
	)
	.parse(input)
}

fn string(input: &str) -> PResult<str, String, ()> {
	let p = fold(
		choice((
			literal("\\\"").map_out(|_| '\"'),
			literal("\\\\").map_out(|_| '\\'),
			literal("\\/").map_out(|_| '/'),
			literal("\\b").map_out(|_| '\x08'),
			literal("\\f").map_out(|_| '\x0C'),
			literal("\\n").map_out(|_| '\n'),
			literal("\\r").map_out(|_| '\r'),
			literal("\\t").map_out(|_| '\t'),
			none_of_char(&['\\', '"']),
		)),
		String::new(),
		|acc, ch| acc.push(ch),
	)
	.map_err(|_| ());

	delimited(literal_char('"').map_err(|_| ()), p, literal_char('"').map_err(|_| ()))
		.parse(input)
}

fn array(input: &str) -> PResult<str, Vec<Value>, ()> {
	let p = value.before(option(literal_char(',')).map_err(|_| ()));
	let folded = fold(p, Vec::new(), |acc, value| acc.push(value));
	let delimited = delimited(
		literal_char('[').map_err(|_| ()),
		folded,
		literal_char(']').map_err(|_| ()),
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
