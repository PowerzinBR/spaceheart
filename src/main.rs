use ansi_term::Colour;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};

struct Config {
    directory_color: Colour,
    prompt_character: String,
}

impl Config {
    fn new() -> Config {
        let default_directory_color = Colour::RGB(255, 51, 51); // red
        let default_prompt_character = String::from("❯");

        Config {
            directory_color: default_directory_color,
            prompt_character: default_prompt_character,
        }
    }
}

fn main() {
    let config = Config::new();

    let mut aliases: Vec<String> = Vec::new();
    if let Some(home_dir) = dirs::home_dir() {
        if let Ok(file) = File::open(home_dir.join(".bashrc")) {
            read_aliases_from_file(&mut aliases, file);
        }
        if let Ok(file) = File::open(home_dir.join(".config/fish/config.fish")) {
            read_aliases_from_file(&mut aliases, file);
        }
    }

    loop {
        let current_dir = env::current_dir().unwrap_or_default();
        let current_dir_str = current_dir.to_str().unwrap_or("");

        let home_dir = dirs::home_dir().unwrap_or_default();
        let home_dir_str = home_dir.to_str().unwrap_or("");
        let current_dir_str = current_dir_str.replace(home_dir_str, "~");

        let directory_line = format!("{}", config.directory_color.paint(current_dir_str));
        let prompt_line = format!("{}", config.prompt_character);

        print!("{} ", directory_line);
        print!("{} ", prompt_line);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
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

        let mut child = Command::new(command)
            .args(&args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to execute command");

        child.wait().expect("Command execution failed");
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

fn read_aliases_from_file(aliases: &mut Vec<String>, file: File) {
    let reader = BufReader::new(file);
    for line in reader.lines() {
        if let Ok(line) = line {
            if line.starts_with("alias") {
                let parts: Vec<&str> = line.split('=').collect();
                if let Some(alias) = parts.get(0) {
                    aliases.push(alias.trim().to_string());
                }
            }
        }
    }
}

fn find_alias_command<'a>(
    input: &'a str,
    aliases: &'a [String],
) -> Option<Box<dyn BuiltinCommand + 'a>> {
    let command = input.split_whitespace().next().unwrap_or("");
    for alias in aliases {
        if alias.starts_with("alias ") && alias.contains(command) {
            let cmd_parts: Vec<&str> = alias.split('=').collect();
            if let Some(cmd) = cmd_parts.get(1) {
                let command = cmd.trim().to_string();
                return match command.split_whitespace().next() {
                    Some("cd") => Some(Box::new(CdCommand::new(
                        command.trim_start_matches("cd").trim(),
                    ))),
                    _ => None,
                };
            }
        }
    }
    None
}
