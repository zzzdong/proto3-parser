use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("io error: {:?}", source))]
    IoError { source: std::io::Error },

    #[snafu(display("grammar error"))]
    GrammarError {
        source: pest::error::Error<crate::parser::Rule>,
    },

    #[snafu(display("parse int error: {:?}", source))]
    ParseIntError { source: std::num::ParseIntError },

    #[snafu(display("unexpect token: {:?}", token))]
    UnexpectToken { token: String, location: String },

    #[snafu(display("token not found: {:?}", token))]
    TokenNotFound { token: String },
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoError { source: e }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Error {
        Error::ParseIntError { source: e }
    }
}

impl From<pest::error::Error<crate::parser::Rule>> for Error {
    fn from(e: pest::error::Error<crate::parser::Rule>) -> Error {
        Error::GrammarError { source: e }
    }
}

pub(crate) fn unexpect_token(pair: pest::iterators::Pair<'_, crate::parser::Rule>) -> Error {
    let span = pair.as_span();
    Error::UnexpectToken {
        token:  span.as_str().to_string(),
        location: format!("{:?}", span.start_pos().line_col()),
    }
}

pub(crate) fn token_not_found(token: impl ToString) -> Error {
    Error::TokenNotFound {
        token: token.to_string(),
    }
}

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;
