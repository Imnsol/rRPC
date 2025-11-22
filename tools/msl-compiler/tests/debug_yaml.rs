use std::path::PathBuf;
use std::fs;

#[test]
fn print_position_value() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("../../examples/schema/workspace.msl");
    let s = fs::read_to_string(&p).expect("read msl");
    let v: serde_yaml::Value = serde_yaml::from_str(&s).expect("parse yaml");
    println!("types raw: {:#?}", v);
    let node_pos = &v["types"]["Node"]["position"];
    println!("Node.position raw repr: {:#?}", node_pos);
}
