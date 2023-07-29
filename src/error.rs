use thiserror::Error;

/// Represents an error that can occur when reading an osu! file.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to parse file: {:?}", .0)]
    Parser(#[from] nom::Err<nom::error::Error<Vec<u8>>>),

    #[error("I/O error occurred: {:?}", .0)]
    IO(#[from] std::io::Error),
}
