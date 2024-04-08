use ansi_term::Color;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::Editor;
use std::env;
use std::process::{Command, Stdio};

struct Config {
    directory_color: Color,
    prompt_character: String,
}

fn main() {
    let rl_result = Editor::<(), FileHistory>::new();
    if let Ok(mut rl) = rl_result {
        if let Err(_) = rl.load_history(".spaceheart_history") {
            eprintln!("No previous history.");
        }

        let aliases: Vec<String> = Vec::new();

        loop {
            let current_dir = env::current_dir().unwrap_or_default();
            let current_dir_str = current_dir.to_str().unwrap_or("");

            let home_dir = dirs::home_dir().unwrap_or_default();
            let home_dir_str = home_dir.to_str().unwrap_or("");
            let current_dir_str = current_dir_str.replace(home_dir_str, "~");

            let config = Config {
                directory_color: Color::Blue,
                prompt_character: "❯".to_string(),
            };

            let directory_line = format!("{}", config.directory_color.paint(current_dir_str));
            let prompt_line = format!("{}", config.prompt_character);

            let readline = rl.readline(&format!("{} {} ", directory_line, prompt_line));
            match readline {
                Ok(input) => {
                    if let Err(_) = rl.add_history_entry(input.clone()) {
                        eprintln!("Failed to add history entry.");
                    }

                    let input = input.trim();

                    if input == "exit" {
                        break;
                    }

                    if input.is_empty() {
                        continue;
                    }

                    if let Some(alias_cmd) = find_alias_command(&input, &aliases) {
                        if let Err(err) = alias_cmd.execute() {
                            eprintln!("Failed to execute command: {}", err);
                        }
                        continue;
                    }

                    let mut parts = input.split_whitespace();
                    let command = parts.next().unwrap_or("");
                    let args: Vec<&str> = parts.collect();

                    let result = Command::new(command)
                        .args(&args)
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .spawn();

                    match result {
                        Ok(mut child) => {
                            if let Err(err) = child.wait() {
                                eprintln!("Failed to execute command: {}", err);
                            }
                        }
                        Err(err) => {
                            eprintln!("Failed to execute command: {}", err);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }

        if let Err(err) = rl.save_history(".spaceheart_history") {
            eprintln!("Failed to save history: {}", err);
        }
    } else {
        eprintln!("Failed to create Editor instance.");
    }
}

trait BuiltinCommand {
    fn execute(&self) -> Result<(), String>;
}

struct CdCommand {
    directory: String,
}

impl CdCommand {
    fn new(directory: &str) -> Self {
        CdCommand {
            directory: directory.to_string(),
        }
    }
}

impl BuiltinCommand for CdCommand {
    fn execute(&self) -> Result<(), String> {
        if let Err(err) = env::set_current_dir(&self.directory) {
            return Err(format!("Failed to change directory: {}", err));
        }
        Ok(())
    }
}

fn find_alias_command<'a>(
    input: &'a str,
    aliases: &'a [String],
) -> Option<Box<dyn BuiltinCommand + 'a>> {
    let command = input.split_whitespace().next().unwrap_or("");
    for alias in aliases {
        if alias.starts_with("alias ") {
            let parts: Vec<&str> = alias.split('=').collect();
            if let Some(alias_cmd) = parts.get(1) {
                let alias_cmd = alias_cmd.trim().trim_matches('\'');
                let alias_parts: Vec<&str> = alias_cmd.split_whitespace().collect();
                if alias_parts.len() > 0 && alias_parts[0] == command {
                    let args = alias_parts[1..].to_vec();
                    return Some(match alias_parts[0] {
                        "cd" => Box::new(CdCommand::new(&args.join(" "))),
                        _ => continue,
                    });
                }
            }
        }
    }
    None
}
