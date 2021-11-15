use std::fmt::{self, Display, Formatter};
use std::error::Error;
use syn::Error as SynError;
use std::io::Error as IoError;
use candid::Error as CandidError;

#[derive(Debug)]
pub enum ProcessingError {
    Io(IoError),
    Syn(SynError),
    Candid(CandidError),
}

impl Display for ProcessingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => err.fmt(f),
            Self::Syn(err) => err.fmt(f),
            Self::Candid(err) => err.fmt(f),
        }
    }
}

impl Error for ProcessingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Syn(err) => Some(err),
            Self::Candid(err) => Some(err),
        }
    }
}

impl From<IoError> for ProcessingError {
    fn from(err: IoError) -> Self {
        Self::Io(err)
    }
}

impl From<SynError> for ProcessingError {
    fn from(err: SynError) -> Self {
        Self::Syn(err)
    }
}

impl From<CandidError> for ProcessingError {
    fn from(err: CandidError) -> Self {
        Self::Candid(err)
    }
}
