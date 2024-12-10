mod ast_util;
mod generate_module;
mod jib_util;

use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;
use common::{intern, HashMap};
use log::info;
use sailrs::{init_logger, parse_sail_files};

use ast_util::{Ast, AST};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Logging level as string (e.g. "debug" or "trace")
    #[arg(long)]
    loglv: Option<String>,

    /// Json file path contains target sail files.
    input: PathBuf,

    /// Output path
    output: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    // parse command line arguments
    let args = Args::parse();

    // set up the logger, defaulting to no output if the CLI flag was not supplied
    init_logger(args.loglv.as_deref().unwrap_or("info"))?;

    intern::init(HashMap::default());

    AST.set(Ast::new(parse_sail_files(args.input).unwrap()))
        .unwrap();

    ast_util::instruction::show_encoding_rule("riscv_insts_zbb.sail");
    ast_util::csrs::show_csrs_definition("riscv_csr_begin.sail");

    generate_module::create_hikami_module(args.output).unwrap();

    info!("done");

    Ok(())
}
