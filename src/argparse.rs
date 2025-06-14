use std::env::Args;

pub enum Commands {
    Init,
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
}

pub fn parse_args(mut args: Args) -> Option<Commands> {
    let prog_name = args.next().unwrap_or("rsafe".to_string());
    
    match args.next().as_deref() {
        Some("version") => show_version(),

        Some("init") => return Some(Commands::Init),

        Some("add") => {
            if let Some(entry) = args.next() {
                return Some(Commands::Add(entry)); 
            }
            println!("[?] Missing argument for 'add'");
        },
        
        Some("edit") => {
            if let Some(entry) = args.next() {
                return Some(Commands::Edit(entry));
            }
            println!("[?] Missing arguments for 'edit'");
        },
        
        Some("get") => {
            if let Some(entry) = args.next() {
                return Some(Commands::Get(entry));
            }
            println!("[?] Missing argument for 'get'");
        },

        Some("list") => return Some(Commands::List),

        Some("rm") => {
            if let Some(entry) = args.next() {
                return Some(Commands::Delete(entry));
            }
            println!("[?] Missing argument for 'rm'");
        },

        Some("generate") => {
            if let Some(size) = args.next() {
                let size: usize = match size.parse() {
                    Ok(x) => x,
                    Err(_) => {
                        println!("[?] Not a valid number!");
                        return None;
                    }
                };
                return Some(Commands::Generate(size));
            }
            println!("[?] Missing argument for 'generate'");
        },

        Some("passwd") => return Some(Commands::Passwd),

        Some("import") => {
            if let Some(path) = args.next() {
                return Some(Commands::Import(path));
            }
            println!("[?] Missing argument for 'import'");
        },

        Some("export") => return Some(Commands::Export),
    
        Some("help") => helper(prog_name, Commands::Help),
        
        Some(unknown) => helper(prog_name, Commands::Invalid(unknown.to_string())),

        None => helper(prog_name, Commands::Help),
    }

    None
}

fn helper(prog_name: String, command: Commands) {
    if let Commands::Invalid(cmd) = command {
        println!("✘ Unknown command '{}'\nTry '{} help' for usage.", cmd, prog_name);
        return;
    }
    
    if let Commands::Help = command {
    println!(
r#"Usage:
  {} <command> [options]

Commands:
  init               Initiate's the database
  add <name>         Add a new password entry
  get <name>         Retrieve a password
  list               List all saved entries
  rm <name>          Remove an entry
  generate <size>    Generate a secure password
  passwd             Change master password
  import <path>      import passwords from a file
  export             export saved passwords to a file"#, prog_name);
    }
}

fn show_version() {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
