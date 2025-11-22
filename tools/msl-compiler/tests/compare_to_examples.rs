use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn generated_matches_handcrafted_examples() {
    let td = tempdir().expect("create tmp");
    let out = td.path().to_path_buf();

    // input path (two levels up from crate to repo examples)
    let mut input = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    input.push("../../examples/schema/workspace.msl");
    let input = input.canonicalize().expect("canonicalize input");

    msl_compiler::compile_schema(&input, &out).expect("compile ok");

    // Compare a few key files against the checked-in examples/schema-generated
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/schema-generated").canonicalize().unwrap();

    let files = vec![
        (out.join("fsharp/Generated.fs"), root.join("fsharp/Generated.fs")),
        (out.join("rust/src/lib.rs"), root.join("rust/src/lib.rs")),
        (out.join("go/node.go"), root.join("go/node.go")),
        (out.join("ts/node.ts"), root.join("ts/node.ts")),
    ];

    for (a,b) in files {
        let left = std::fs::read_to_string(&a).expect(&format!("read gen {}", a.display()));
        let right = std::fs::read_to_string(&b).expect(&format!("read example {}", b.display()));
        assert_eq!(left.trim(), right.trim(), "{} != {}", a.display(), b.display());
    }
}
