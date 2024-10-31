use std::{
    fmt::Display,
    process::{ExitCode, Termination},
};

#[derive(PartialEq, Eq, Debug)]
pub struct AppError {
    pub message: String,
}

impl From<&str> for AppError {
    fn from(err: &str) -> AppError {
        AppError {
            message: String::from(err),
        }
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        AppError {
            message: value.clone(),
        }
    }
}

impl Termination for AppError {
    fn report(self) -> ExitCode {
        ExitCode::FAILURE
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
