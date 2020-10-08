use crate::compiler::{Value, Error};
use std::rc::Rc;

pub fn method(method_name: &str) -> Option<&'static dyn Fn(&String, &[Value]) -> Result<Value, Error>> {
    Some(match method_name {
        "parse_json" => &parse_json,
        "parse_yaml" => &parse_yaml,
        "parse_toml" => &parse_toml,
        _ => return None,
    })
}

fn parse_json(string: &String, args: &[Value]) -> Result<Value, Error> {
    ensure!(args.len() == 0, "'parse_json' does not take any arguments");
    let x = serde_json::from_str(string.as_str())
        .map_err(|e| anyhow!("cannot parse JSON: {}", e))?;
    Ok(Value::HashMap(Rc::new(x)))
}

fn parse_yaml(string: &String, args: &[Value]) -> Result<Value, Error> {
    ensure!(args.len() == 0, "'parse_yaml' does not take any arguments");
    let x = serde_yaml::from_str(string.as_str())
        .map_err(|e| anyhow!("cannot parse YAML: {}", e))?;
    Ok(Value::HashMap(Rc::new(x)))
}

fn parse_toml(string: &String, args: &[Value]) -> Result<Value, Error> {
    ensure!(args.len() == 0, "'parse_toml' does not take any arguments");
    let x = toml::de::from_str(string.as_str())
        .map_err(|e| anyhow!("cannot parse TOML: {}", e))?;
    Ok(Value::HashMap(Rc::new(x)))
}

