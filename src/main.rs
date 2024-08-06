use std::{env, fs, path::PathBuf};

use clap::{Parser, Subcommand};
use javaver::config::{self, JavaverConfig, SDKConfig};

const CONFIG_PATH: &str = "javaver-config.json";

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {

    #[command(subcommand)]
    command: Option<Commands>,

}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Automatically search for SDKs to add in directories
    Auto {

        search_path: Option<PathBuf>,

    },
    /// Manually add an SDK
    Add {

        name: String,
        path: PathBuf,

    },
    /// Select added SDKs as current
    Sel {

        name: String,

    },
    /// Remove an already-added SDK
    Rm {

        name: String,

    },
    /// List all added SDKs
    List,
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
    let mut config;
    let config_path = PathBuf::from(CONFIG_PATH);

    if config_path.exists() {
        match config::read_config(&config_path) {
            Ok(c) => {
                config = c;
            }
            Err(err) => {
                println!("Error while reading config - quitting...\nError: {}", err);
                return;
            }
        }
    } else {
        config = JavaverConfig::new();
    }

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Auto { search_path }) => {
            let mut paths = vec![
                PathBuf::from("C:\\Program Files\\Java"), 
                PathBuf::from("C:\\Program Files\\Eclipse Adoptium"),
            ];
            
            if let Some(path) = search_path {
                paths.push(path.into());
            }

            for path in paths.iter() {
                println!("Searching in: {}", path.display());
                if !path.exists() {
                    println!("Path does not exist. Skipping...");
                    continue;
                }

                let subpaths = match fs::read_dir(path) {
                    Ok(subpaths) => subpaths,
                    Err(err) => {
                        println!("Error encountered while trying to fetch subdirectories: {}\nSkipping...", err);
                        continue;
                    }
                };

                for subpath in subpaths {
                    if subpath.is_err() {
                        println!("Encountered an erroneous subdirectory entry: {}\nSkipping...", subpath.unwrap_err());
                        continue;
                    }

                    let subpath = subpath.unwrap();
                    if !validate_java_path(&subpath.path()) {
                        println!("Subdirectory is not a Java SDK path. Skipping...");
                        continue;
                    }

                    let name = subpath.file_name().into_string().unwrap();
                    if config.contains_name(name.as_str()) {
                        println!("SDK under the name '{}' has already been added. Please choose a different name and add the SDK manually.", name);
                        continue;
                    }

                    config.sdk.push(SDKConfig::new(&name, &subpath.path()));
                    println!("Successfully added: '{}' under the name '{}'.", subpath.path().display(), name);
                }
            }
        }
        Some(Commands::Add { name, path }) => {
            if !validate_java_path(path) {
                return;
            }

            if config.contains_name(name.as_str()) {
                println!("The name is already in use.");
                return;
            }

            config.sdk.push(SDKConfig::new(name, path));
        }
        Some(Commands::Sel { name }) => {
            if !config.contains_name(name) {
                println!("There is no SDK added named '{}'", name);
                return;
            }

            let sdk = config.sdk.iter().find(|sdk| &sdk.name == name).unwrap();
            let path = &sdk.path;

            println!("{}", env::var("Path").unwrap());

            env::set_var("JAVA_HOME", path.to_str().unwrap());
            println!("Successfully selected '{}' as current Java SDK", name);
        },
        Some(Commands::Rm { name }) => {

        }
        Some(Commands::List) => {
            if config.sdk.is_empty() {
                println!("There are no SDKs added.");
            } else {
                println!("Displaying all SDKs:");
            }

            for sdk in config.sdk.iter() {
                println!("{}: {}", sdk.name, sdk.path.to_str().unwrap());
            }
        }
        None => {
            println!("No arguments were provided. Quitting...\nRun `javaver --help` to see the list of options.");
        }
    }

    if let Err(err) = config::write_config(&config, &config_path) {
        println!("Failed to write to config. All changes have not been saved.\nError: {}", err);
    }
}
