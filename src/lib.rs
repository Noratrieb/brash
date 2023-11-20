pub mod parse;

use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};

pub fn bash_it(args: impl Iterator<Item = String>) -> Result<()> {
    let (filename, src) = parse_args_into_src(args)?;

    parse::parse(filename, &src)?;

    Ok(())
}

fn parse_args_into_src(args: impl Iterator<Item = String>) -> Result<(Arc<str>, String)> {
    let mut src = None;
    let mut c = false;

    for arg in args {
        if src.is_some() {
            bail!("usage: brash [FILE]");
        }
        if c {
            src = Some(("<cmd>".into(), arg));
        } else if arg == "-c" {
            c = true;
        } else {
            src = Some((
                arg.clone().into(),
                std::fs::read_to_string(&arg).with_context(|| format!("opening {arg}"))?,
            ));
        }
    }

    src.ok_or_else(|| anyhow!("usage: brash [FILE]"))
}
