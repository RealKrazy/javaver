use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {

        name: String,

        path: PathBuf,

    },
    Sel {

        name: String,

    },
}

fn validate_java_path(path: &PathBuf) -> bool {
    if !path.exists() {
        println!("Provided path doesn't exist.");
        return false;
    }

    let java_path = path.join("bin\\java.exe");
    if !java_path.exists() {
        println!("Provided path is not a JDK installation.");
        return false;
    }

    true
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Add { name, path }) => {
            if !validate_java_path(path) {
                return;
            }
        }
        Some(Commands::Sel { name }) => {

        }
        None => {
            println!("No arguments were provided. Please run `javaver --help` to see the list of options.");
        }
    }
}
