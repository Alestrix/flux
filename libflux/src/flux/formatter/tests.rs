use super::*;
use crate::parser::Parser;
use std::str;

// This gives us a colorful diff.
#[cfg(test)]
use pretty_assertions::assert_eq;

fn format_helper(golden: &str) {
    let file = Parser::new(golden).parse_file("".to_string());
    let mut fmt = Formatter::new(golden.len());
    fmt.format_file(&file, true);
    let (output, _) = fmt.output();
    assert_eq!(golden, output);
}

fn format_modified(input: &str, expected: &str) {
    let file = Parser::new(input).parse_file("".to_string());
    let mut fmt = Formatter::new(input.len());
    fmt.format_file(&file, true);
    let (output, _) = fmt.output();
    assert_eq!(expected, output);
}

#[test]
fn binary_op() {
    format_helper("1 + 1 - 2");
    format_helper("1 * 1 / 2");
    format_helper("2 ^ 4");
    format_helper("1 * (1 / 2)");
}

#[test]
fn funcs() {
    format_helper(
        r#"(r) =>
	(r.user == "user1")"#,
    );
    format_helper(
        r#"add = (a, b) =>
	(a + b)"#,
    ); // decl
    format_helper("add(a: 1, b: 2)"); // call
    format_helper(
        r#"foo = (arg=[]) =>
	(1)"#,
    ); // nil value as default
    format_helper(
        r#"foo = (arg=[1, 2]) =>
	(1)"#,
    ); // none nil value as default
}

#[test]
fn object() {
    format_helper("{a: 1, b: {c: 11, d: 12}}");
    format_helper("{foo with a: 1, b: {c: 11, d: 12}}"); // with
    format_helper("{a, b, c}"); // implicit key object literal
    format_helper(r#"{"a": 1, "b": 2}"#); // object with string literal keys
    format_helper(r#"{"a": 1, b: 2}"#); // object with mixed keys
}

#[test]
fn member() {
    format_helper("object.property"); // member ident
    format_helper(r#"object["property"]"#); // member string literal
}

#[test]
fn array() {
    format_helper(
        r#"a = [1, 2, 3]

a[i]"#,
    );
}

#[test]
fn conditional() {
    format_helper("if a then b else c");
    format_helper(r#"if not a or b and c then 2 / (3 * 2) else obj.a(par: "foo")"#);
}

#[test]
fn return_expr() {
    format_helper("return 42");
}

#[test]
fn option() {
    format_helper("option foo = {a: 1}");
    format_helper(r#"option alert.state = "Warning""#); // qualified
}

#[test]
fn vars() {
    format_helper("0.1"); // float
    format_helper("100000000.0"); // integer float
    format_helper("365d"); // duration
    format_helper("1d1m1s"); // duration_multiple
    format_helper("2018-05-22T19:53:00Z"); // time
    format_helper("2018-05-22T19:53:01+07:00"); // zone
    format_helper("2018-05-22T19:53:23.09012Z"); // nano sec
    format_helper("2018-05-22T19:53:01.09012-07:00"); // nano with zone
    format_helper(r#"/^\w+@[a-zA-Z_]+?\.[a-zA-Z]{2,3}$/"#); // regexp
    format_helper(r#"/^http:\/\/\w+\.com$/"#); // regexp_escape
}

#[test]
fn block() {
    format_helper(
        r#"foo = () => {
	foo(f: 1)
	1 + 1
}"#,
    );
}

#[test]
fn str_lit() {
    format_helper(r#""foo""#);
    format_helper(
        r#""this is
a string
with multiple lines""#,
    ); // multi lines
    format_helper(r#""foo \\ \" \r\n""#); // with escape
    format_helper(r#""\xe6\x97\xa5\xe6\x9c\xac\xe8\xaa\x9e""#); // with byte
}

#[test]
fn package_import() {
    format_helper(
        r#"package foo
"#,
    ); // package
    format_helper(
        r#"import "path/foo"
import bar "path/bar""#,
    ); // imports
    format_helper(
        r#"import foo "path/foo"

foo.from(bucket: "testdb")
	|> range(start: 2018-05-20T19:53:26Z)"#,
    ); // no_package
    format_helper(
        r#"package foo


from(bucket: "testdb")
	|> range(start: 2018-05-20T19:53:26Z)"#,
    ); // no_imports
    format_helper(
        r#"package foo


import "path/foo"
import bar "path/bar"

from(bucket: "testdb")
	|> range(start: 2018-05-20T19:53:26Z)"#,
    ); // package import
}

#[test]
fn simple() {
    format_helper(
        r#"from(bucket: "testdb")
	|> range(start: 2018-05-20T19:53:26Z)
	|> filter(fn: (r) =>
		(r.name =~ /.*0/))
	|> group(by: ["_measurement", "_start"])
	|> map(fn: (r) =>
		({_time: r._time, io_time: r._value}))"#,
    );
}

#[test]
fn medium() {
    format_helper(
        r#"from(bucket: "testdb")
	|> range(start: 2018-05-20T19:53:26Z)
	|> filter(fn: (r) =>
		(r.name =~ /.*0/))
	|> group(by: ["_measurement", "_start"])
	|> map(fn: (r) =>
		({_time: r1._time, io_time: r._value}))"#,
    )
}

#[test]
fn complex() {
    format_helper(
        r#"left = from(bucket: "test")
	|> range(start: 2018-05-22T19:53:00Z, stop: 2018-05-22T19:55:00Z)
	|> drop(columns: ["_start", "_stop"])
	|> filter(fn: (r) =>
		(r.user == "user1"))
	|> group(by: ["user"])
right = from(bucket: "test")
	|> range(start: 2018-05-22T19:53:00Z, stop: 2018-05-22T19:55:00Z)
	|> drop(columns: ["_start", "_stop"])
	|> filter(fn: (r) =>
		(r.user == "user2"))
	|> group(by: ["_measurement"])

join(tables: {left: left, right: right}, on: ["_time", "_measurement"])"#,
    );
}

#[test]
fn option_complete() {
    format_helper(
        r#"option task = {
	name: "foo",
	every: 1h,
	delay: 10m,
	cron: "02***",
	retry: 5,
}

from(bucket: "test")
	|> range(start: 2018-05-22T19:53:26Z)
	|> window(every: task.every)
	|> group(by: ["_field", "host"])
	|> sum()
	|> to(bucket: "test", tagColumns: ["host", "_field"])"#,
    )
}

#[test]
fn functions_complete() {
    format_helper(
        r#"foo = () =>
	(from(bucket: "testdb"))
bar = (x=<-) =>
	(x
		|> filter(fn: (r) =>
			(r.name =~ /.*0/)))
baz = (y=<-) =>
	(y
		|> map(fn: (r) =>
			({_time: r._time, io_time: r._value})))

foo()
	|> bar()
	|> baz()"#,
    )
}

#[test]
fn multi_indent() {
    format_helper(
        r#"_sortLimit = (n, desc, columns=["_value"], tables=<-) =>
	(tables
		|> sort(columns: columns, desc: desc)
		|> limit(n: n))
_highestOrLowest = (n, _sortLimit, reducer, columns=["_value"], by, tables=<-) =>
	(tables
		|> group(by: by)
		|> reducer()
		|> group(none: true)
		|> _sortLimit(n: n, columns: columns))
highestAverage = (n, columns=["_value"], by, tables=<-) =>
	(tables
		|> _highestOrLowest(
			n: n,
			columns: columns,
			by: by,
			reducer: (tables=<-) =>
				(tables
					|> mean(columns: [columns[0]])),
			_sortLimit: top,
		))"#,
    )
}

#[test]
fn comments() {
    format_helper("// attach to id\nid");
    format_helper("// attach to int\n1");
    format_helper("// attach to float\n1.1");
    format_helper("// attach to string\n\"hello\"");
    format_helper("// attach to regex\n/hello/");
    format_helper("// attach to time\n2020-02-28T00:00:00Z");
    format_helper("// attach to duration\n2m");
    format_helper("// attach to bool\ntrue");
    format_modified(
        "// attach to open paren\n( 1 + 1 )",
        "// attach to open paren\n1 + 1",
    );
    format_modified(
        "( 1 + 1 // attach to close paren\n )",
        "1 + 1\n\t// attach to close paren\n\t",
    );
    // A reordering we have to live with, unless we do some refactoring in the
    // formatter.
    format_modified(
        "1 * // attach to open paren\n( 1 + 1 )",
        "1 * (\n\t// attach to open paren\n\t1 + 1)",
    );
    format_helper("1 * (1 + 1\n\t// attach to close paren\n\t)");
    format_helper("from\n\t//comment\n\t(bucket: bucket)");
    format_helper("from(\n\t//comment\n\tbucket: bucket)");
    format_helper("from(bucket\n\t//comment\n\t: bucket)");
    format_helper("from(bucket: \n\t//comment\n\tbucket)");
    format_helper("from(bucket: bucket\n\t//comment\n\t)");
    format_helper("from(\n\t//comment\n\tbucket)");
    format_helper("from(bucket\n\t//comment\n\t, _option)");
    format_helper("from(bucket, \n\t//comment\n\t_option)");
    format_helper("from(bucket, _option\n\t//comment\n\t)");
    format_modified(
        "from(bucket, _option//comment1\n,//comment2\n)",
        "from(bucket, _option\n\t//comment1\n\t//comment2\n)",
    );

    /* Expressions. */
    format_helper("1 \n\t//comment\n\t<= 1");
    format_helper("1 \n\t//comment\n\t+ 1");
    format_helper("1 \n\t//comment\n\t* 1");
    format_helper("from()\n\t//comment\n\t|> to()");
    format_helper("//comment\n+1");
    format_modified("1 * //comment\n-1", "1 * (\n\t//comment\n\t-1)");
    format_helper("i = \n\t//comment\n\tnot true");
    format_helper("//comment\nexists 1");
    format_helper("a \n\t//comment\n\t=~ /foo/");
    format_helper("a \n\t//comment\n\t!~ /foo/");
    format_helper("a \n\t//comment\n\tand b");
    format_helper("a \n\t//comment\n\tor b");

    format_helper("a\n\t//comment\n\t = 1");
    format_helper("//comment\noption a = 1");
    format_helper("option a\n\t//comment\n\t = 1");
    format_helper("option a\n\t//comment\n\t.b = 1");
    format_helper("option a.b\n\t//comment\n\t = 1");

    // Some funny business here. Propbably need to scan write_string for \n
    format_helper("f = \n\t//comment\n\t(a) =>\n\t(a())");
    format_helper("f = (\n\t//comment\n\ta) =>\n\t(a())");
    format_helper("f = (\n\t//comment\n\ta, b) =>\n\t(a())");
    format_helper("f = (a\n\t//comment\n\t, b) =>\n\t(a())");
    format_helper("f = (a\n\t//comment\n\t=1, b=2) =>\n\t\t(a())");
    format_helper("f = (a=\n\t//comment\n\t1, b=2) =>\n\t(a())");
    format_helper("f = (a=1\n\t//comment\n\t, b=2) =>\n\t\t(a())");
    format_helper("f = (a=1, \n\t//comment\n\tb=2) =>\n\t\t(a())");
    format_helper("f = (a=1, b\n\t//comment\n\t=2) =>\n\t\t(a())");
    format_helper("f = (a=1, b=\n\t//comment\n\t2) =>\n\t(a())");
    format_modified(
        "f = (a=1, b=2//comment\n,) =>\n\t(a())",
        "f = (a=1, b=2\n\t//comment\n\t) =>\n\t(a())",
    );
    format_helper("f = (a=1, b=2\n\t//comment\n\t) =>\n\t(a())");
    format_modified(
        "f = (a=1, b=2,//comment\n) =>\n\t(a())",
        "f = (a=1, b=2\n\t//comment\n\t) =>\n\t(a())",
    );
    format_modified(
        "f = (a=1, b=2//comment1\n,//comment2\n) =>\n\t(a())",
        "f = (a=1, b=2\n\t//comment1\n\t//comment2\n\t) =>\n\t(a())",
    );
    format_helper("f = (a=1, b=2) \n\t//comment\n\t=>\n\t(a())");
    format_modified(
        "f = (a=1, b=2) =>\n\t//comment\n(a())",
        "f = (a=1, b=2) =>\n\t(\n\t\t//comment\n\t\ta())",
    );
    format_modified(
        "f = (a=1, b=2) =>\n\t//comment\na()",
        "f = (a=1, b=2) =>\n\t(\n\t\t//comment\n\t\ta())",
    );

    format_helper("//comment\ntest a = 1");
    format_helper("test \n\t//comment\n\ta = 1");
    format_helper("test a\n\t//comment\n\t = 1");
    format_helper("test a = \n\t//comment\n\t1");

    format_helper("//comment\nreturn x");
    format_helper("return \n\t//comment\n\tx");

    format_helper("//comment\nif 1 then 2 else 3");
    format_helper("if \n\t//comment\n\t1 then 2 else 3");
    format_helper("if 1\n\t//comment\n\t then 2 else 3");
    format_helper("if 1 then \n\t//comment\n\t2 else 3");
    format_helper("if 1 then 2\n\t//comment\n\t else 3");
    format_helper("if 1 then 2 else \n\t//comment\n\t3");

    format_helper("//comment\nfoo[\"bar\"]");
    format_helper("foo\n\t//comment\n\t[\"bar\"]");
    format_helper("foo[\n\t//comment\n\t\"bar\"]");
    format_helper("foo[\"bar\"\n\t//comment\n\t]");

    format_helper("a = \n\t//comment\n\t[1, 2, 3]");
    format_helper("a = [\n\t//comment\n\t1, 2, 3]");
    format_helper("a = [1\n\t//comment\n\t, 2, 3]");
    format_helper("a = [1, \n\t//comment\n\t2, 3]");
    format_helper("a = [1, \n\t//comment1\n\t2\n\t//comment2\n\t, 3]");
    format_helper("a = [1, 2, 3\n\t//comment\n\t]");

    format_helper("a = b\n\t//comment\n\t[1]");
    format_helper("a = b[\n\t//comment\n\t1]");
    format_helper("a = b[1\n\t//comment\n\t]");

    format_helper("//comment\n{_time: r._time, io_time: r._value}");
    format_helper("{\n\t//comment\n\t_time: r._time, io_time: r._value}");
    format_helper("{_time\n\t//comment\n\t: r._time, io_time: r._value}");
    format_helper("{_time: \n\t//comment\n\tr._time, io_time: r._value}");
    // Missing preservation in front of '.'
    // format_helper("{_time: r\n\t//comment\n\t._time, io_time: r._value}");
    format_helper("{_time: r.\n\t//comment\n\t_time, io_time: r._value}");
    format_helper("{_time: r._time\n\t//comment\n\t, io_time: r._value}");
    format_helper("{_time: r._time, \n\t//comment\n\tio_time: r._value}");
    format_helper("{_time: r._time, io_time\n\t//comment\n\t: r._value}");
    format_helper("{_time: r._time, io_time: \n\t//comment\n\ttr._value}");
    // Missing preservation in front of '.'
    // format_helper("{_time: r._time, io_time: r\n\t//comment\n\t._value}");
    format_helper("{_time: r._time, io_time: r.\n\t//comment\n\t_value}");
    format_helper("{_time: r._time, io_time: r._value\n\t//comment\n\t}");
    format_modified(
        "{_time: r._time, io_time: r._value\n\t//comment\n\t,}",
        "{_time: r._time, io_time: r._value\n\t//comment\n\t}",
    );
    format_modified(
        "{_time: r._time, io_time: r._value,\n\t//comment\n\t}",
        "{_time: r._time, io_time: r._value\n\t//comment\n\t}",
    );

    format_helper("//comment\nimport \"foo\"");
    format_helper("import \n\t//comment\n\t\"foo\"");
    format_helper("import \n\t//comment\n\tfoo \"foo\"");

    format_helper("//comment\npackage foo\n");
    format_helper("package \n\t//comment\n\tfoo\n");

    format_helper("{\n\t//comment\n\tfoo with a: 1, b: 2}");
    format_helper("{foo\n\t//comment\n\t with a: 1, b: 2}");
    format_helper("{foo with \n\t//comment\n\ta: 1, b: 2}");

    format_helper("fn = (tables=\n\t//comment\n\t<-) =>\n\t(tables)");

    // Comments around braces needs some work.
    format_helper("fn = (a) => \n\t//comment\n\t{\n\treturn a\n}");
    format_helper("fn = (a) => {\n\treturn a\n// ending\n}");

    format_modified(
        r#"    // hi
// there
{_time: r._time, io_time: r._value, // this is the end
}

// minimal
foo = (arg=[1, 2]) => (1)

// left
left = from(bucket: "test")
	|> range(start: 2018-05-22T19:53:00Z
	// i write too many comments
	, stop: 2018-05-22T19:55:00Z)
	// and put them in strange places
	|>  drop

		// this hurts my eyes
(columns: ["_start", "_stop"])
		// just terrible
	|> filter(fn: (r) =>
		(r.user 

		// (don't fire me, this is intentional)
		== "user1"))
	|> group(by
	// strange place for a comment
: ["user"])

right = from(bucket: "test")
	|> range(start: 2018-05-22T19:53:00Z,
			// please stop
			stop: 2018-05-22T19:55:00Z)
	|> drop( // spare me the pain
// this hurts
columns: ["_start", "_stop"// what
])
	|> filter(
		// just why
		fn: (r) =>
		// user 2 is the best user
		(r.user == "user2"))
	|> group(by: //just stop
["_measurement"])

join(tables: {left: left, right: right}, on: ["_time", "_measurement"])

from(bucket, _option // friends
,// stick together
)

i = // definitely
not true
// a
// list
// of
// comments

j

// not lost
"#,
        r#"// hi
// there
{_time: r._time, io_time: r._value
	// this is the end
	}

// minimal
foo = (arg=[1, 2]) =>
	(1)
// left
left = from(bucket: "test")
	|> range(start: 2018-05-22T19:53:00Z
		// i write too many comments
		, stop: 2018-05-22T19:55:00Z)
	// and put them in strange places
	|> drop
		// this hurts my eyes
		(columns: ["_start", "_stop"])
	// just terrible
	|> filter(fn: (r) =>
		(r.user 
			// (don't fire me, this is intentional)
			== "user1"))
	|> group(by
		// strange place for a comment
		: ["user"])
right = from(bucket: "test")
	|> range(start: 2018-05-22T19:53:00Z, 
		// please stop
		stop: 2018-05-22T19:55:00Z)
	|> drop(
		// spare me the pain
		// this hurts
		columns: ["_start", "_stop"
			// what
			])
	|> filter(
		// just why
		fn: (r) =>
		(
			// user 2 is the best user
			r.user == "user2"))
	|> group(by: 
		//just stop
		["_measurement"])

join(tables: {left: left, right: right}, on: ["_time", "_measurement"])
from(bucket, _option
	// friends
	// stick together
)

i = 
	// definitely
	not true

// a
// list
// of
// comments
j
// not lost
"#,
    );
}
