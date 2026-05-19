use clap::{Parser, Subcommand};
use glass_driver::pipeline;
use std::path::PathBuf;

fn print_header() {
    println!();
    println!("~~~~~~~~~~~~~~~~~~");
    println!("|                |");
    println!("| Glass Compiler |");
    println!("|                |");
    println!("~~~~~~~~~~~~~~~~~~");
    println!();
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile just 1 file. This may not generate a working executable - mostly for testing
    CompileOne {
        /// the filename to compile
        #[arg()]
        filename: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    print_header();

    // if cli.debug {
    //     println!("Debug mode is kind of on")
    // } else {
    //     println!("Debug mode is off")
    // }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::CompileOne { filename } => match filename.canonicalize() {
            Ok(path) => {
                println!("Running for filename: {:?}", path);
                println!();

                match pipeline(path.to_str().unwrap()) {
                    Ok(_result) => {
                        println!("Complied Successfully");
                    }
                    Err(_) => {
                        println!("Failed To Compile");
                    }
                }
            }
            Err(e) => {
                println!("{e}: {filename:?}");
            }
        },
    }
}
