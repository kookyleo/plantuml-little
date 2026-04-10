use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse {
        line: usize,
        column: Option<usize>,
        message: String,
    },
    JavaErrorPage {
        line: usize,
        message: String,
    },
    UnsupportedDiagram(String),
    Render(String),
    Layout(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {e}"),
            Error::Parse {
                line,
                column: Some(col),
                message,
            } => write!(f, "Parse error at line {line}:{col}: {message}"),
            Error::Parse {
                line,
                column: None,
                message,
            } => write!(f, "Parse error at line {line}: {message}"),
            Error::JavaErrorPage { line, message } => {
                write!(f, "Java-style error page at line {line}: {message}")
            }
            Error::UnsupportedDiagram(s) => write!(f, "Unsupported diagram type: {s}"),
            Error::Render(s) => write!(f, "Render error: {s}"),
            Error::Layout(s) => write!(f, "Layout error: {s}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
