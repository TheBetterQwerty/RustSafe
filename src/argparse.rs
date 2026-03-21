#![allow(unused_assignments)]
#![allow(unused)]

use std::env::Args;

#[derive(Debug)]
pub enum Commands {
    Init,
    Logs,
    Add(String),            // Add new entry
    Get(String),            // Get record based on username or email
    List,                   // Shows all entries
    Edit(String),           // Edits the entered record based on username or email
    Delete(String),          // Deletes a entry
    Generate(usize),          // Generates a password of 'n' size
    Passwd,                  // change master password
    Export,                 // Exports to file
    Import(String),         // Imports from given path
    Help,                   // Prints help
    Invalid(String),        // Invalid command

    /* Profile Manipulation */
    Default(String),
    CreateProfile(String),
    EditProfile(String),
    DeleteProfile(String),
    ListProfiles
}

pub fn parse_args(mut args: Args) -> Option<(Option<String>,Commands)> {
    let prog_name = args.next().unwrap_or("rsafe".to_string());
    let (mut profile, mut command) = (None, None);

    let missing_cmd = |x: &str|
        println!("[!] Missing argument for '{}'. Try {} --help", x, prog_name);

    while let Some(cmd) = args.next().as_deref() {
        match cmd {
            /* Profile Manipulation */
            "--set-profile-default" => {
                if let Some(profile_name) = args.next() {
                    command = Some(Commands::Default(profile_name));
                    break;
                }
                missing_cmd(cmd);
            },

            "--create-profile" => {
                if let Some(profile_name) = args.next() {
                    command = Some(Commands::CreateProfile(profile_name));
                    break;
                }
                missing_cmd(cmd);
            },

            "--edit-profile" => {
                if let Some(profile_name) = args.next() {
                    command = Some(Commands::EditProfile(profile_name));
                    break;
                }
                missing_cmd(cmd);
            },

            "--delete-profile" => {
                if let Some(profile_name) = args.next() {
                    command = Some(Commands::DeleteProfile(profile_name));
                    break;
                }
                missing_cmd(cmd);
            },

            "--list-profiles" => {
                command = Some(Commands::ListProfiles);
                break;
            },

            /* ******************************************** */

            "--from" => {
                if let Some(profile_name) = args.next() {
                    profile = Some(profile_name);
                    continue;
                }
                missing_cmd(cmd);
            },

            "--init" => {
                command = Some(Commands::Init);
                break;
            },

            "--add" => {
                if let Some(arg) = args.next() {
                    command = Some(Commands::Add(arg));
                    continue;
                }
                missing_cmd(cmd);
            },

            "--get" => {
                if let Some(arg) = args.next() {
                    command = Some(Commands::Get(arg));
                    continue;
                }
                missing_cmd(cmd);
            },

            "--list" => {
                command = Some(Commands::List);
                continue;
            },

            "--logs" => {
                command = Some(Commands::Logs);
                continue;
            },

            "--edit" => {
                if let Some(arg) = args.next() {
                    command = Some(Commands::Edit(arg));
                    continue;
                }
                missing_cmd(cmd);
            },

            "--rm" => {
                if let Some(arg) = args.next() {
                    command = Some(Commands::Delete(arg));
                    continue;
                }
                missing_cmd(cmd);
            },

            "--generate" | "-g" => {
                let size = args.next().as_deref()
                    .unwrap_or("30")
                    .parse::<usize>()
                    .expect("Error: String to number!");

                command = Some(Commands::Generate(size));
            },

            "--passwd" => {
                command = Some(Commands::Passwd);
                continue;
            },

            "--import" => {
                if let Some(arg) = args.next() {
                    command = Some(Commands::Import(arg));
                    continue;
                }
                missing_cmd(cmd);
            },

            "--export" => {
                command = Some(Commands::Export);
                continue;
            },

            _ => {}
        }
    }

    if command.is_none() {
        helper(prog_name, Commands::Help);
        return None;
    }

    Some((profile, command.unwrap()))
}

fn helper(prog_name: String, command: Commands) {
    if let Commands::Invalid(cmd) = command {
        let valid_cmds: Vec<&str> = vec![
            "--version", "--init", "--add", "--get", "--list", "--logs",
            "--edit", "--rm", "--generate", "--passwd", "--import", "--export",
            "--set-default-profile", "--from", "--create-profile", "--edit-profile",
            "--delete-profile", "--list-profiles"
        ];

        for valid_cmd in valid_cmds.iter() {
            if (*valid_cmd).contains(&cmd) {
                println!("Unknown command '{}'\nSimilar Command '{}' exists", cmd, valid_cmd);
                return;
            }
        }

        println!("Unknown command '{}'\nTry '{} --help' for usage.", cmd, prog_name);
        return;
    }

    if let Commands::Help = command {
        println!("Usage:");
        println!("  {} <command> [options]", prog_name);

        println!("\nCommands:");
        println!("  --version                     Displays current version");
        println!("  --init                        Initiates the database");
        println!("  --logs                        Prints the saved logs");
        println!("  --add <name>                  Add a new password entry");
        println!("  --get <name>                  Retrieve a password");
        println!("  --list                        List all saved entries");
        println!("  --edit <name>                 Edit an entry");
        println!("  --rm <name>                   Remove an entry");
        println!("  --generate <size>             Generate a secure password");
        println!("  --passwd                      Change master password");
        println!("  --import <path>               Import passwords from a file");
        println!("  --export                      Export saved passwords to a file");

        println!("\nProfile Commands:");
        println!("  --create-profile <name>       Create a new profile");
        println!("  --edit-profile <name>         Rename or modify a profile");
        println!("  --delete-profile <name>       Delete a profile");
        println!("  --list-profiles               List all profiles");
        println!("  --set-default-profile <name>  Set default profile");

        println!("\nProfile Usage:");
        println!("  --from <name>                 Execute command using specified profile");
    }
}

fn show_version() {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
