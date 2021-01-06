use std::fs::File;
use std::io::{Read, Write};
use std::{env, io};

use clap::{App, Arg};
use tracing_subscriber::EnvFilter;

use syconf_lib::SerializableValue;

fn main() {
    let matches = App::new("syconf")
        .version(env!("CARGO_PKG_VERSION"))
        .about("syconf converts syconf files into JSON/YAML/TOML")
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
                .possible_values(&["json", "yaml", "yaml-stream", "toml", "text"])
                .default_value("json"),
        )
        .get_matches();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let result = match matches.value_of("input").unwrap() {
        "-" => {
            let mut s = String::new();
            io::stdin().read_to_string(&mut s).unwrap();
            syconf_lib::parse_string(&s)
        }
        file => syconf_lib::parse_file(file),
    };

    let val = match result {
        Ok(val) => val.to_serializable(),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    };

    let ser = match matches.value_of("format").unwrap() {
        "json" => serde_json::to_string(&val).unwrap(),
        "yaml" => serde_yaml::to_string(&val).unwrap(),
        "yaml-stream" => to_yaml_stream(&val),
        "toml" => toml::ser::to_string(&val).unwrap(),
        "text" => {
            if let SerializableValue::String(s) = val {
                s.to_string()
            } else {
                eprintln!("ERROR: resulting error is not a string");
                std::process::exit(1);
            }
        }
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

fn to_yaml_stream(val: &SerializableValue) -> String {
    match val {
        SerializableValue::List(list) => list
            .iter()
            .map(|x| serde_yaml::to_string(x).unwrap())
            .collect::<Vec<String>>()
            .join("\n\n"),
        v => serde_yaml::to_string(v).unwrap(),
    }
}
