// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
use std::thread::{self, JoinHandle};
use thiserror::Error;

#[derive(Debug, Error)]
enum DetectorError {
    #[error("An error occurred: {0}")]
    SomeError(String),
}

fn detector() -> JoinHandle<Result<(), DetectorError>> {
    thread::spawn(|| Ok(()))
}
