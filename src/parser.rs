use crate::parser::BuiltinCmd::{Cd, Echo, Exit, Pwd, Type};

pub enum ShellCmd {
    Builtin(BuiltinCmd),
    // Empty,
    Program(String),
}

pub enum BuiltinCmd {
    Cd,
    Echo,
    Exit,
    Pwd,
    Type,
}

// pub struct ProgramArgs {
//     pub(crate) fname: String,
//     pub(crate) args: Vec<String>,
// }

// TODO: n2 this "parse" has become so barebones that i don't know if it's worth it anymore...
// maybe a HasMap of string to function for builtins would make more sense...? not sure
// but eventuall i want to support quoting, pipe, etc. so maybe we still need this...?
pub fn parse_cmd_name(cmd: &str) -> ShellCmd {
    match cmd {
        "cd" => ShellCmd::Builtin(Cd),
        "exit" => ShellCmd::Builtin(Exit),
        "echo" => ShellCmd::Builtin(Echo),
        "pwd" => ShellCmd::Builtin(Pwd),
        "type" => ShellCmd::Builtin(Type),
        fname => ShellCmd::Program(fname.to_string()),
    }
}
