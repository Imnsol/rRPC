use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn generated_outputs_basic_checks() {
    let td = tempdir().unwrap();
    let out = td.path().to_path_buf();

    let mut input = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crate is under tools/msl-compiler, examples live at repo/examples
    input.push("../../examples/schema/workspace.msl");
    // normalize path
    let input = input.canonicalize().unwrap();

    let mut outdir = out.clone();

    // call compile_schema from library
    msl_compiler::compile_schema(&input, &outdir).expect("compile should succeed");

    // check fsharp
    let mut fsharp = outdir.clone(); fsharp.push("fsharp/Generated.fs");
    let f = std::fs::read_to_string(&fsharp).expect("read fsharp output");
    assert!(f.contains("type Node"));
    assert!(f.contains("Id: Guid") || f.contains("Id: System.Guid"));
    assert!(f.contains("HyperEdge"));

    // check rust
    let mut rr = outdir.clone(); rr.push("rust/src/lib.rs");
    let r = std::fs::read_to_string(&rr).expect("read rust output");
    assert!(r.contains("pub struct Node"));
    assert!(r.contains("position") && r.contains("[f64; 4]") || r.contains("Vec<f64>"));
    assert!(r.contains("HyperEdge"));

    // check go
    let mut gg = outdir.clone(); gg.push("go/node.go");
    let g = std::fs::read_to_string(&gg).expect("read go output");
    assert!(g.contains("Position [4]float64") || g.contains("Position []float64"));
    // Label may be a pointer (*string), dynamic array ([]string) or plain string
    assert!(g.contains("Label *string") || g.contains("Label []string") || g.contains("Label string"));

    // check ts
    let mut ts = outdir.clone(); ts.push("ts/node.ts");
    let t = std::fs::read_to_string(&ts).expect("read ts output");
    assert!(t.contains("export interface Node"));
    assert!(t.contains("position") || t.contains("position"));
    assert!(t.contains("label") );
}
