use std::process::ExitStatus;

use crate::parse::Command;

pub fn execute(cmds: &[Command<'_>]) {
    for cmd in cmds {
        execute_command(cmd);
    }
}

fn execute_command(cmd: &Command<'_>) -> ExitStatus {
    match cmd {
        Command::Single(cmd) => {
            if cmd.exclamation {
                todo!()
            }
            if cmd.time {
                todo!()
            }
            if cmd.terminator.is_some() {
                todo!()
            }
            if !cmd.rest.is_empty() {
                todo!()
            }
            let cmds = &cmd.initial;
            if !cmds.rest.is_empty() {
                todo!()
            }

            let program = &cmds.initial.tokens[0].value;
            let mut proc = std::process::Command::new(program.as_ref());
            proc.args(
                cmds.initial.tokens[1..]
                    .iter()
                    .map(|t| -> &str { &t.value.as_ref() }),
            );

            let mut proc = proc.spawn().unwrap();

            let status = proc.wait().unwrap();

            status
        }
        Command::Subshell(_) => todo!(),
        Command::Block(_) => todo!(),
    }
}
