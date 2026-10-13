use regex::Regex;
#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        command = command.trim().to_string();
        let parsed = parse_cmd(&command);
        if parsed.cmd == "exit" {
            break;
        } else if parsed.cmd == "echo" {
            println!("{}", parsed.args.join(" "))
        } else if parsed.cmd == "type" {
            let first_arg = parsed.args[0].as_str();
            match first_arg {
                "echo"|"type"|"exit" => {
                    println!("{} is a shell builtin", first_arg)
                }
                _ => {
                    println!("{}: not found", first_arg)
                }
            }
        } else {
            println!("{}: command not found", command);
        }
        // if cmd_is_valid(&command) {
        //     println!("Command is valid");
        // } else {
        //     println!("{}: command not found", command.trim());
        // }
        io::stdout().flush().unwrap();
    }
}

struct ParsedCommand {
    cmd: String,
    args: Vec<String>,
}

fn parse_cmd(cmd: &str) -> ParsedCommand {
    let ws = Regex::new(r#"\s+"#).unwrap();
    let tokens: Vec<&str> = ws.split(cmd).collect();
    let cmd = tokens[0].to_string();
    let args: Vec<String> = tokens[1..tokens.len()]
        .iter()
        .map(|x| x.to_string())
        .collect();
    ParsedCommand { cmd, args }
}

fn cmd_is_valid(cmd: &str) -> bool {
    // cmd.trim() != ""
    false
}
