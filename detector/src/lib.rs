#[cfg(feature = "hikvision")]
include!(concat!(env!("OUT_DIR"), "/hikcamera/camera.rs"));

#[cfg(feature = "mindvision")]
include!(concat!(env!("OUT_DIR"), "hikcamera/camera.rs"));

use std::thread::{self, JoinHandle};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DetectorError {
    #[error("An error occurred: {0}")]
    SomeError(String),
}

pub fn detector() -> JoinHandle<Result<(), DetectorError>> {
    unsafe {
        let mut res: u32 = 0;
        init();
        enumerate_devices((&mut res) as *mut u32);
        println!("{}", res);
        final_();
    }
    thread::spawn(|| Ok(()))
}
