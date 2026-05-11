use std::io::{Write, stderr};

use dust_lang::{error::Error as DustError, source::SourceError};

pub enum Error<'src> {
    Dust(DustError<'src>),
    Io(std::io::Error),
    Ron(ron::Error),
    Toml(toml::de::Error),
}

impl Error<'_> {
    /// Prints the error and returns an exit code.
    pub fn finish(&self) -> i32 {
        match self {
            Error::Dust(dust_error) => {
                let count = dust_error.error_count();

                if count == 1 {
                    write_error_message("\n1 error found.");
                } else {
                    write_error_message(&format!("\n{count} errors found."));
                }

                1
            }
            Error::Io(io_error) => {
                write_error_message(&io_error.to_string());

                io_error.raw_os_error().unwrap_or(1)
            }
            Error::Ron(ron_error) => {
                write_error_message(&ron_error.to_string());

                1
            }
            Error::Toml(toml_error) => {
                write_error_message(&toml_error.to_string());

                1
            }
        }
    }
}

impl<'src> From<DustError<'src>> for Error<'src> {
    fn from(error: DustError<'src>) -> Self {
        Error::Dust(error)
    }
}

impl<'src> From<SourceError> for Error<'src> {
    fn from(error: SourceError) -> Self {
        Error::Dust(DustError::from(error))
    }
}

impl From<std::io::Error> for Error<'_> {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<ron::Error> for Error<'_> {
    fn from(error: ron::Error) -> Self {
        Error::Ron(error)
    }
}

impl From<toml::de::Error> for Error<'_> {
    fn from(error: toml::de::Error) -> Self {
        Error::Toml(error)
    }
}

fn write_error_message(message: &str) {
    match stderr().write_all(message.as_bytes()) {
        Ok(()) => {}
        Err(error) => {
            Error::Io(error).finish();
        }
    }
}
