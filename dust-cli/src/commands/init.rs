use crate::{cli::InitCommand, error::Error};

pub fn init<'src>(command: InitCommand) -> Result<(), Error<'src>> {
    let InitCommand { path: _, global: _ } = command;

    todo!("Re-implement the init command in a library so that it can emit better errors.");
}
