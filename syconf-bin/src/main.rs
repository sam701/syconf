use std::fs::File;
use std::io;
use std::io::{Read, Write};

use clap::{App, Arg};
use tracing::Level;
use std::collections::BTreeMap;
use syconf_lib::Value;
use std::rc::Rc;

fn main() {
    let matches = App::new("syconf")
        .version("0.1.0")
        .about("syconf converts syconf files into JSON/YAML/TOML")
        .arg(Arg::with_name("debug")
            .long("debug")
            .short("d")
            .help("Turn on debug output"))
        .arg(Arg::with_name("input")
            .long("input")
            .short("i")
            .help("Input file name")
            .takes_value(true)
            .value_name("FILE")
            .default_value("stdin"))
        .arg(Arg::with_name("output")
            .long("output")
            .short("o")
            .help("Output file name")
            .takes_value(true)
            .value_name("FILE")
            .default_value("stdout"))
        .arg(Arg::with_name("format")
            .long("format")
            .short("f")
            .help("Output format")
            .takes_value(true)
            .value_name("FORMAT")
            .possible_values(&["json", "yaml", "toml"])
            .default_value("json"))
        .get_matches();

    if matches.is_present("debug") {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .init();
    }

    let result =
        match matches.value_of("input").unwrap() {
            "stdin" => {
                let mut s = String::new();
                io::stdin().read_to_string(&mut s).unwrap();
                syconf_lib::parse_string(&s)
            }
            file => syconf_lib::parse_file(file)
        }.map(to_serializable);


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
        _ => unreachable!()
    };

    match matches.value_of("output").unwrap() {
        "stdout" => io::stdout().write_all(ser.as_bytes()).unwrap(),
        file => File::create(file).unwrap().write_all(ser.as_bytes()).unwrap(),
    }
}

#[derive(serde::Serialize)]
#[serde(untagged)]
enum SerializableValue {
    Bool(bool),
    Int(i32),
    String(String),
    HashMap(BTreeMap<String, SerializableValue>),
    List(Vec<SerializableValue>),
}

fn to_serializable(v: Value) -> SerializableValue {
    match v {
        Value::Bool(x) => SerializableValue::Bool(x),
        Value::Int(x) => SerializableValue::Int(x),
        Value::String(x) => SerializableValue::String(Rc::try_unwrap(x).unwrap()),
        Value::HashMap(x) => SerializableValue::HashMap(Rc::try_unwrap(x).unwrap().into_iter().map(|(k, v)| (k, to_serializable(v))).collect()),
        Value::List(x) => SerializableValue::List(Rc::try_unwrap(x).unwrap().into_iter().map(to_serializable).collect()),
        Value::Func(x) => SerializableValue::String("<function>".to_string()),
    }
}