extern crate winapi;

use std::collections::HashMap;
use std::io;
use std::{env, ffi::OsString, fs, iter, path::PathBuf};
use std::os::windows::ffi::OsStrExt;

use clap::{Parser, Subcommand};
use dialoguer::Select;
use javaver::config::{self, JavaverConfig, SDKConfig};
use winapi::shared::minwindef::LPARAM;
use winapi::um::winuser;
use winreg::RegKey;

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

        name: Option<String>,

    },
    /// Remove an already-added SDK from the list
    Rm {

        name: String,

    },
    /// List all added SDKs
    List,
}

fn get_exe_dir() -> io::Result<PathBuf> {
    let mut path = env::current_exe()?;
    path.pop();

    Ok(path)
}

fn encode_win_string(str: &str) -> Vec<u16> {
    OsString::from(str)
    .encode_wide()
    .chain(iter::once(0))
    .collect()
}

fn read_system_vars() -> HashMap<String, String> {
    let hkcu = RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let env = hkcu.open_subkey_with_flags("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment", winreg::enums::KEY_READ)
        .expect("Failed to open registry keys");

    let mut env_vars = HashMap::new();
    for (name, value) in env.enum_values().map(|x| x.unwrap()) {
        env_vars.insert(name, value.to_string());
    }

    env_vars
}

fn read_system_path_var() -> Vec<String> {
    let vars = read_system_vars();
    let path = vars.iter().find(|el| el.0.to_lowercase() == "path").unwrap();

    path.1.split(';').map(|entry| entry.to_string()).collect()
}

fn set_system_path_var(value: &str) -> io::Result<()> {
    let hkcu = RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let env = hkcu.open_subkey_with_flags("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment", winreg::enums::KEY_WRITE)?;

    env.set_value("Path", &value)
}

fn notify_env_change() {
    let event = encode_win_string("Environment");

    unsafe {
        winuser::SendMessageTimeoutW(
            winuser::HWND_BROADCAST,
            winuser::WM_SETTINGCHANGE,
            0,
            event.as_ptr() as LPARAM,
            winuser::SMTO_ABORTIFHUNG,
            5000,
            std::ptr::null_mut(),
        );
    }
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

fn select_dialoguer(config: &JavaverConfig) {
    let list = config.sdk.iter().map(|sdk| sdk.name.as_str()).collect::<Vec<&str>>();

    let select = Select::new()
        .with_prompt("Choose existing SDK from the list")
        .items(&list)
        .interact()
        .unwrap();

    select_named(config, &config.sdk.get(select).unwrap().name);
}

fn select_named(config: &JavaverConfig, name: &str) {
    if !config.contains_name(name) {
        println!("There is no SDK added named '{}'", name);
        return;
    }

    let sdk = config.sdk.iter().find(|sdk| &sdk.name == name).unwrap();
    let path = sdk.path.to_str().unwrap();
    let path_bin = &sdk.path.join("bin");
    let path_bin = path_bin.to_str().unwrap();

    let mut path_vars = read_system_path_var();
    if let Some(pos) = path_vars.iter().position(|el| el.to_lowercase() == path_bin.to_lowercase()) {
        path_vars.remove(pos);
    }

    path_vars.insert(0, path_bin.to_owned());

    if let Err(err) = set_system_path_var(path_vars.join(";").as_str()) {
        println!("Error when modifying system-wide 'Path' environment variable: {}\nYou might not be running as an administrator.", err);
        return;
    }
    
    notify_env_change();

    env::set_var("JAVA_HOME", path);
    println!("Successfully selected '{}' as current Java SDK", name);
}

fn main() {
    let mut config;
    let config_path = get_exe_dir().unwrap().join(PathBuf::from(CONFIG_PATH));

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
            match name {
                Some(name) => select_named(&config, name.as_str()),
                None => select_dialoguer(&config),
            }
        },
        Some(Commands::Rm { name }) => {
            if !config.contains_name(name.as_str()) {
                println!("There is no SDK under the name '{}'.", name);
                return;
            }

            let pos = config.sdk.iter().position(|el| el.name == name.to_owned()).unwrap();
            config.sdk.remove(pos);

            println!("Successfully removed '{}' from the SDK list.", name);
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
