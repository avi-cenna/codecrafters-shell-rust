#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        command = command.trim().to_string();
        if command == "exit" {
            break;
        };
        println!("{}: command not found", command);
        // if cmd_is_valid(&command) {
        //     println!("Command is valid");
        // } else {
        //     println!("{}: command not found", command.trim());
        // }
        io::stdout().flush().unwrap();
    }
}

fn cmd_is_valid(cmd: &str) -> bool {
    // cmd.trim() != ""
    false
}
