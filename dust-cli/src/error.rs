use std::fmt::{self, Display, Formatter};

use dust_lang::{error::Error as DustError, source::SourceError};

pub enum Error<'src> {
    Dust(DustError<'src>),
    Io(std::io::Error),
    Ron(ron::Error),
    Toml(toml::de::Error),
}

impl Display for Error<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::Dust(dust_error) => write!(f, "{dust_error}"),
            Error::Io(io_error) => write!(f, "{io_error}"),
            Error::Ron(ron_error) => write!(f, "{ron_error}"),
            Error::Toml(toml_error) => write!(f, "{toml_error}"),
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
