mod cli;

use cli::Cli;

use std::fs::{metadata, File};
use std::io::{self, ErrorKind, Read};

use clap::Parser;

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let mut path = cli.file;

    path = path.canonicalize()?;

    if !path.is_file() {
        return io::Result::Err(io::Error::new(
            ErrorKind::Other,
            format!("Provided path {path:?} does not point to a file!"),
        ));
    }

    let len = metadata(path.clone())?.len();

    let mut bytecode = Vec::with_capacity(len as usize);

    let mut file = File::open(path.clone())?;

    file.read_to_end(&mut bytecode)?;

    dbg!(bytecode);

    Ok(())
}
