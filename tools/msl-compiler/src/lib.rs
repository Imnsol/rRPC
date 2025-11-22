use std::fs;
use std::path::PathBuf;
use serde::Deserialize;
use anyhow::{Result, Context as AnyhowContext};

#[derive(Debug, Deserialize)]
pub struct Schema {
    pub schema: Option<String>,
    pub types: Option<serde_yaml::Mapping>,
    pub ui: Option<serde_yaml::Mapping>,
}

pub fn compile_schema(input: &PathBuf, out_dir: &PathBuf) -> Result<()> {
    let s = fs::read_to_string(input).with_context(|| format!("read {}", input.display()))?;
    let schema: Schema = serde_yaml::from_str(&s)?;

    let types_map = schema.types.unwrap_or_default();

    let mut types = vec![];
    for (k, v) in types_map.iter() {
        let type_name = k.as_str().unwrap_or_default().to_string();
        let fields = if let serde_yaml::Value::Mapping(m) = v { m.clone() } else { serde_yaml::Mapping::new() };
        types.push((type_name, fields));
    }

    fs::create_dir_all(out_dir.join("fsharp"))?;
    fs::create_dir_all(out_dir.join("rust/src"))?;
    fs::create_dir_all(out_dir.join("go"))?;
    fs::create_dir_all(out_dir.join("ts"))?;

    // Generate F# types
    let mut fsharp = String::new();
    fsharp.push_str("namespace Schema\n\nopen System\nopen System.Text.Json\nopen System.Text.Json.Serialization\n\n");
    for (name, fields) in &types {
            fsharp.push_str(&format!("[<CLIMutable>]\ntype {} = {{\n", name));
        for (fk, fv) in fields.iter() {
            let fname = fk.as_str().unwrap_or_default();
            let ftype = yaml_value_to_fsharp_type(fv);
            fsharp.push_str(&format!("    {}: {}\n", title_case(fname), ftype));
        }
        fsharp.push_str("}\n\n");
    }
    fsharp.push_str("module Codec =\n    let serialize<'T> (x: 'T) = JsonSerializer.SerializeToUtf8Bytes(x)\n    let deserialize<'T> (b: byte[]) : 'T = JsonSerializer.Deserialize<'T>(b)\n");
    fs::write(out_dir.join("fsharp/Generated.fs"), fsharp)?;

    // Generate Rust types (simple)
    let mut rust = String::new();
    rust.push_str("use serde::{Serialize, Deserialize};\n\n");
    // outside test functions add types
    let mut types_str = String::new();
    for (name, fields) in &types {
            types_str.push_str(&format!("#[derive(Debug, Serialize, Deserialize, PartialEq)]\npub struct {} {{\n", name));
        for (fk, fv) in fields.iter() {
            let fname = fk.as_str().unwrap_or_default();
            let ftype = yaml_value_to_rust_type(fv);
            types_str.push_str(&format!("    pub {}: {},\n", fname.to_lowercase(), ftype));
        }
        types_str.push_str("}\n\n");
    }
    rust.push_str(&types_str);
    rust.push_str("#[cfg(test)]\nmod tests { use super::*; use serde_json; use uuid;\n\n    #[test]\n    fn roundtrip_dummy() {\n        // generation test left intentionally minimal for prototype\n    }\n}\n");
    fs::write(out_dir.join("rust/src/lib.rs"), rust)?;

    // Generate Go
    let mut go = String::new();
    go.push_str("package schema\n\nimport \"encoding/json\"\n\n");
    for (name, fields) in &types {
            go.push_str(&format!("type {} struct {{\n", name));
        for (fk, fv) in fields.iter() {
            let fname = fk.as_str().unwrap_or_default();
            let ftype = yaml_value_to_go_type(fv);
            // if optional (pointer) add omitempty
            if ftype.starts_with('*') {
                // use plain type (non-pointer) but add omitempty in tag to indicate optionality
                let plain = ftype.trim_start_matches('*');
                go.push_str(&format!("    {} {} `json:\"{},omitempty\"`\n", title_case(fname), plain, fname));
            } else {
                go.push_str(&format!("    {} {} `json:\"{}\"`\n", title_case(fname), ftype, fname));
            }
        }
        go.push_str("}\n\n");
    }
    fs::write(out_dir.join("go/node.go"), go)?;

    // Generate TS types
    let mut ts = String::new();
    for (name, fields) in &types {
            ts.push_str(&format!("export interface {} {{\n", name));
        for (fk, fv) in fields.iter() {
            let fname = fk.as_str().unwrap_or_default();
            let mut ftype = yaml_value_to_ts_type(fv);
            // if the TS generator used `| undefined` produce optional property `name?: T` instead
            if ftype.contains("| undefined") {
                ftype = ftype.replace(" | undefined", "");
                ts.push_str(&format!("  {}?: {};\n", fname, ftype));
            } else {
                ts.push_str(&format!("  {}: {};\n", fname, ftype));
            }
        }
        ts.push_str("}\n\n");
    }
    fs::write(out_dir.join("ts/node.ts"), ts)?;

    Ok(())
}

fn title_case(s: &str) -> String {
    if s.is_empty() { return s.to_string(); }
    let mut c = s.chars();
    match c.next() { Some(f) => f.to_uppercase().to_string() + c.as_str(), None => String::new() }
}

fn yaml_value_to_fsharp_type(v: &serde_yaml::Value) -> String {
    let s = match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Sequence(seq) => {
            let parts: Vec<String> = seq.iter().map(|e| match e {
                serde_yaml::Value::String(s) => s.clone(),
                other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
            }).collect();
            format!("[{}]", parts.join(";"))
        }
        other => serde_yaml::to_string(other).unwrap_or_default(),
    };
    let optional = s.trim().ends_with('?');
    let base = s.trim_end_matches('?').trim();

    if base.starts_with('[') && base.ends_with(']') {
        let inner = &base[1..base.len()-1];
        if inner.contains(';') {
            let parts: Vec<_> = inner.split(';').map(|p| p.trim()).collect();
            let t = parts.get(0).unwrap_or(&"");
            let ftyp = match *t {
                "f64" => "float[]",
                "uuid" => "Guid[]",
                "string" => "string[]",
                _ => "obj[]",
            };
            return if optional { format!("{} option", ftyp) } else { ftyp.into() };
        }

        let t = inner.trim();
        let ftyp = match t {
            "f64" => "float[]",
            "uuid" => "Guid[]",
            "string" => "string[]",
            _ => "obj[]",
        };
        return if optional { format!("{} option", ftyp) } else { ftyp.into() };
    }

    let mapped = match base {
        "uuid" => "Guid",
        "string" => "string",
        "f64" => "float",
        _ => "obj",
    };
    return if optional { format!("{} option", mapped) } else { mapped.into() };
}

fn yaml_value_to_rust_type(v: &serde_yaml::Value) -> String {
    let s = match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Sequence(seq) => {
            let parts: Vec<String> = seq.iter().map(|e| match e {
                serde_yaml::Value::String(s) => s.clone(),
                other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
            }).collect();
            format!("[{}]", parts.join(";"))
        }
        other => serde_yaml::to_string(other).unwrap_or_default(),
    };
    let optional = s.trim().ends_with('?');
    let base = s.trim_end_matches('?').trim();

    if base.starts_with('[') && base.ends_with(']') {
        let inner = &base[1..base.len()-1];
        if inner.contains(';') {
            let parts: Vec<_> = inner.split(';').map(|p| p.trim()).collect();
            let t = parts.get(0).unwrap_or(&"");
            let got = match *t {
                "f64" => "[f64; 4]".into(),
                "uuid" => "Vec<uuid::Uuid>".into(),
                "string" => "[String; 4]".into(),
                _ => "Vec<serde_json::Value>".into(),
            };
            return if optional { format!("Option<{}>", got) } else { got };
        }

        let t = inner.trim();
        let got = match t {
            "f64" => "Vec<f64>".into(),
            "uuid" => "Vec<uuid::Uuid>".into(),
            "string" => "Vec<String>".into(),
            _ => "Vec<serde_json::Value>".into(),
        };
        return if optional { format!("Option<{}>", got) } else { got };
    }

    let mapped = match base {
        "uuid" => "uuid::Uuid".into(),
        "string" => "String".into(),
        "f64" => "f64".into(),
        _ => "serde_json::Value".into(),
    };
    return if optional { format!("Option<{}>", mapped) } else { mapped };
}

fn yaml_value_to_go_type(v: &serde_yaml::Value) -> String {
    let s = match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Sequence(seq) => {
            let parts: Vec<String> = seq.iter().map(|e| match e {
                serde_yaml::Value::String(s) => s.clone(),
                other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
            }).collect();
            format!("[{}]", parts.join(";"))
        }
        other => serde_yaml::to_string(other).unwrap_or_default(),
    };
    let optional = s.trim().ends_with('?');
    let base = s.trim_end_matches('?').trim();

    if base.starts_with('[') && base.ends_with(']') {
        let inner = &base[1..base.len()-1];
        if inner.contains(';') {
            let parts: Vec<_> = inner.split(';').map(|p| p.trim()).collect();
            let t = parts.get(0).unwrap_or(&"");
            let got = match *t {
                "f64" => format!("[{}]float64", parts.get(1).unwrap_or(&"4")),
                "uuid" => format!("[]string"),
                "string" => format!("[{}]string", parts.get(1).unwrap_or(&"4")),
                _ => "[]interface{}".into(),
            };
            return if optional { format!("*{}", got) } else { got };
        }

        let t = inner.trim();
        let got = match t {
            "f64" => "[]float64".into(),
            "uuid" => "[]string".into(),
            "string" => "[]string".into(),
            _ => "[]interface{}".into(),
        };
        return if optional { format!("*{}", got) } else { got };
    }

    let mapped = match base {
        "uuid" => "string",
        "string" => "string",
        "f64" => "float64",
        _ => "interface{}",
    };
    return if optional { format!("*{}", mapped) } else { mapped.into() };
}

fn yaml_value_to_ts_type(v: &serde_yaml::Value) -> String {
    let s = match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Sequence(seq) => {
            let parts: Vec<String> = seq.iter().map(|e| match e {
                serde_yaml::Value::String(s) => s.clone(),
                other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
            }).collect();
            format!("[{}]", parts.join(";"))
        }
        other => serde_yaml::to_string(other).unwrap_or_default(),
    };
    let optional = s.trim().ends_with('?');
    let base = s.trim_end_matches('?').trim();

    if base.starts_with('[') && base.ends_with(']') {
        let inner = &base[1..base.len()-1];
        if inner.contains(';') {
            // fixed-size tuple like [f64;4]
            let parts: Vec<_> = inner.split(';').map(|p| p.trim()).collect();
            let t = parts.get(0).unwrap_or(&"");
            let cnt: usize = parts.get(1).and_then(|c| c.parse().ok()).unwrap_or(4);
            let elem = match *t {
                "f64" => "number",
                "uuid" => "string",
                "string" => "string",
                _ => "any",
            };
            let tuple = std::iter::repeat(elem).take(cnt).collect::<Vec<_>>().join(",");
            let got = format!("[{}]", tuple);
            return if optional { format!("{} | undefined", got) } else { got };
        }

        // dynamic array
        let t = inner.trim();
        let got = match t {
            "f64" => "number[]".into(),
            "uuid" => "string[]".into(),
            "string" => "string[]".into(),
            _ => "any[]".into(),
        };
        return if optional { format!("{} | undefined", got) } else { got };
    }

    let mapped = match base {
        "uuid" => "string",
        "string" => "string",
        "f64" => "number",
        _ => "any",
    };
    return if optional { format!("{} | undefined", mapped) } else { mapped.into() };
}
