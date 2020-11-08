use crate::from_str;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Eq, PartialEq, Debug)]
struct Abc {
    name: String,
    age: i32,
    cool: bool,
    nicknames: Vec<String>,
    labels: HashMap<String, String>,
}

#[test]
fn deserialize_struct() {
    let abc: Abc = from_str(
        r#"
        let name = "pooh"
        in
        {
            name: name
            age: 3
            cool: true
            nicknames: ['winnie']
            labels: {
                street: "tree"
            }
        }
    "#,
    )
    .unwrap();
    let mut labels = HashMap::new();
    labels.insert("street".to_owned(), "tree".to_owned());
    assert_eq!(
        abc,
        Abc {
            name: "pooh".to_owned(),
            age: 3,
            cool: true,
            nicknames: vec!["winnie".to_owned()],
            labels,
        }
    )
}
