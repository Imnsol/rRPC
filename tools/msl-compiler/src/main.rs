use std::path::PathBuf;
use structopt::StructOpt;
use anyhow::Result;
use msl_compiler::compile_schema;

#[derive(StructOpt, Debug)]
#[structopt(name = "msl-compiler")]
struct Opt {
    /// Input MSL (YAML) file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output directory for generated code
    #[structopt(parse(from_os_str), short = "o", long = "out", default_value = "examples/schema-generated")]
    out_dir: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    compile_schema(&opt.input, &opt.out_dir)
}
// helpers are implemented in library (src/lib.rs)
