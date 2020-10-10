# SyConf
*SyConf* is a simple configuration language that keeps your configuration lean.

## Why?
**Why another configuration language?**

Software configuration is getting larger and more complex.
Modern software require more complex and large configurations.
Configuration files of Kubernetes, Prometheus, AlertManager, Concourse, Vector.dev, etc. 
tend to get large, repetitive, clumsy, and at the end less maintainable.

Existing configuration languages either do not support templating and functions, like JSON, YAML, TOML, 
or have complex grammars and multiple features that often go beyond the scope of a configuration language, 
e.g. JsonNet, Dhall.

*SyConf* is a simple configuration language that supports used defined functions, string interpolation, 
and helps to keep complex configurations concise and maintainable.

## Features
* Very simple grammar
    ```
    let house_number = 53
    let my_hash = {
        name: "john",
        age: 33,
    }
    in
    {
        person: my_hash,
        address: "Awesome Street ${house_number}",
    }
    ```
* Function support
    ```
    let add2 = (x) => x + 2
    in
    [3, add2(4)]
    ``` 
* String interpolation
    ```
    let name = "alexei"
    in
    "hello ${name}"
    ```
* It is possible to split configuration in multiple files
    ```
    let instance_x = import "./instance_x.sy"
    let instance_y = import "./instance_y.sy"
    in
    {
        instances: [instance_x, instance_y]
    }
    ```
* *SyConf* objects have a small set of meaningful methods
    ```
    let hash = {
        aa: 3,
        bb: 4,
    }
    in
    
    hash.map( (key, value) => [key, value * 10])
    ``` 
* There are helpful functions to interact with the environment
    ```
    let region = getenv("AWS_REGION", "eu-central-1")
    let prom_config = read_file("prometheus.yaml").parse_yaml()
    ```
* Conditionals
    ```
    let x = if getenv("MY_VAR", "default_value") == "abc" then "this" else "that"
    ```
* Block expressions
    ```
    let func = (x) => {
        let a = x + 2
        in
        a * 4
    }
    ```
* There is **no magic** :-)

## Name
The letters `S` and `Y` in the name *SyConf* are the first and the last letter in the word *simplicity*.

## Goal
TODO

## Previous Work
* [JSON](https://www.json.org/json-en.html) - simple, great for machines, not very human friendly.
* [YAML](https://yaml.org) - human friendly, supports entity references, error prone because of sensitivity to indentation.
* [TOML](https://toml.io/en/) - human friendly, eliminates some weaknesses of YAML, big files gets less readable.
* [HCL](https://github.com/hashicorp/hcl) - structured language, supports functions, does not support user defined functions.
* [jsonnet](https://jsonnet.org) - supports functions and templating, has complex syntax, does not support functions as first level citizens.
* [dhall](https://dhall-lang.org) - has support for types, has complex syntax with a lot of features.


## Licence
[Apache 2.0](./LICENSE)
