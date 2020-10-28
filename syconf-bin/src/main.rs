use std::collections::BTreeMap;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::rc::Rc;

use clap::{App, Arg};
use tracing::Level;

use syconf_lib::Value;

fn main() {
    let matches = App::new("syconf")
        .version(env!("CARGO_PKG_VERSION"))
        .about("syconf converts syconf files into JSON/YAML/TOML")
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .short("d")
                .help("Turn on debug output"),
        )
        .arg(
            Arg::with_name("input")
                .help("Input file name")
                .required(true)
                .value_name("CONFIG_FILE"),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
                .help("Output file name")
                .takes_value(true)
                .value_name("FILE")
                .default_value("stdout"),
        )
        .arg(
            Arg::with_name("format")
                .long("format")
                .short("f")
                .help("Output format")
                .takes_value(true)
                .value_name("FORMAT")
                .possible_values(&["json", "yaml", "toml"])
                .default_value("json"),
        )
        .get_matches();

    if matches.is_present("debug") {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .init();
    }

    let result = match matches.value_of("input").unwrap() {
        "-" => {
            let mut s = String::new();
            io::stdin().read_to_string(&mut s).unwrap();
            syconf_lib::parse_string(&s)
        }
        file => syconf_lib::parse_file(file),
    }
    .map(|x| to_serializable(&x));

    let val = match result {
        Ok(val) => val,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    };

    let ser = match matches.value_of("format").unwrap() {
        "json" => serde_json::to_string(&val).unwrap(),
        "yaml" => serde_yaml::to_string(&val).unwrap(),
        "toml" => toml::ser::to_string(&val).unwrap(),
        _ => unreachable!(),
    };

    match matches.value_of("output").unwrap() {
        "stdout" => io::stdout().write_all(ser.as_bytes()).unwrap(),
        file => File::create(file)
            .unwrap()
            .write_all(ser.as_bytes())
            .unwrap(),
    }
}

#[derive(serde::Serialize)]
#[serde(untagged)]
enum SerializableValue {
    Bool(bool),
    Int(i32),
    String(Rc<str>),
    HashMap(BTreeMap<Rc<str>, SerializableValue>),
    List(Rc<[SerializableValue]>),
}

fn to_serializable(v: &Value) -> SerializableValue {
    match v {
        Value::Bool(x) => SerializableValue::Bool(*x),
        Value::Int(x) => SerializableValue::Int(*x),
        Value::String(x) => SerializableValue::String(x.clone()),
        Value::HashMap(x) => SerializableValue::HashMap(
            x.iter()
                .map(|(k, v)| (k.clone(), to_serializable(v)))
                .collect(),
        ),
        Value::List(x) => SerializableValue::List(x.iter().map(to_serializable).collect()),
        Value::Func(_) => SerializableValue::String("<function>".into()),
    }
}
