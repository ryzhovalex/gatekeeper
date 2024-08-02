use std::{env, io, path::PathBuf};

pub fn cwd() -> io::Result<PathBuf> {
    env::current_dir()
}
