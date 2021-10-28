use std::fmt;

#[derive(Debug)]
pub enum AppError {
    NoAPIKey,
    CSV(csv::Error),
    IO(std::io::Error),
    Reqwest(reqwest::Error),
}

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::NoAPIKey => write!(f, "No API key is set via the .env variable"),
            AppError::CSV(err) => write!(f, "Error while writing the CSV file: {}", err),
            AppError::IO(err) => write!(f, "Error while flushing the file: {}", err),
            AppError::Reqwest(err) => write!(f, "Error while fetching data: {}", err),
        }
    }
}

impl From<csv::Error> for AppError {
    fn from(err: csv::Error) -> AppError {
        AppError::CSV(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> AppError {
        AppError::IO(err)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> AppError {
        AppError::Reqwest(err)
    }
}
