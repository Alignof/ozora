mod ast_util;
mod generate_decoder;
mod generate_module;
mod jib_util;

use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;
use common::{intern, HashMap};
use log::info;
use sailrs::{init_logger, parse_sail_files};

use ast_util::{Ast, AST};

const XLEN: u32 = 64;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Logging level as string (e.g. "debug" or "trace")
    #[arg(long)]
    loglv: Option<String>,

    /// Extension name.
    #[arg(long)]
    ext_name: String,

    /// Json file path contains target sail files.
    #[arg(long, default_value = "files.json")]
    input_json: PathBuf,

    /// Target file
    target: PathBuf,

    /// Output path
    output: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    // parse command line arguments
    let args = Args::parse();

    // Check that the output file name matches the extension name.
    assert!(
        args.output
            .file_stem()
            .unwrap()
            .eq_ignore_ascii_case(args.ext_name.as_str()),
        "Extension name does not matches the file name"
    );

    // Check that the first character of the extension name is capitalized
    assert!(
        args.ext_name.chars().next().is_some_and(char::is_uppercase),
        "The First character of the extension name is not capitalized"
    );

    // set up the logger, defaulting to no output if the CLI flag was not supplied
    init_logger(args.loglv.as_deref().unwrap_or("info"))?;

    intern::init(HashMap::default());

    AST.set(Ast::new(parse_sail_files(args.input_json.clone()).unwrap()))
        .unwrap();

    let target_file = args.target.as_os_str().to_str().unwrap();
    let insns = ast_util::instruction::get_encoding_rule(target_file);
    ast_util::csrs::show_csrs_definition(target_file);

    generate_module::create_hikami_module(&args.ext_name, &args.output, &insns).unwrap();

    generate_decoder::instruction_definition::create_raki_insn_def(
        &args.ext_name,
        &args.output,
        &insns,
    )
    .unwrap();

    generate_decoder::parse_operand::create_raki_decoder(&args.ext_name, &args.output, &insns)
        .unwrap();

    info!("done");

    Ok(())
}
