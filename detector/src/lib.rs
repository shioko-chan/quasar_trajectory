#[cfg(feature = "hikvision")]
include!(concat!(env!("OUT_DIR"), "/hikcamera/camera.rs"));

#[cfg(feature = "mindvision")]
include!(concat!(env!("OUT_DIR"), "hikcamera/camera.rs"));

use std::{
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
    thread::{self, JoinHandle},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DetectorError {
    #[error("An error occurred: {0}")]
    SomeError(String),
}

pub fn detector(terminate: Arc<AtomicBool>) -> JoinHandle<Result<(), DetectorError>> {
    unsafe {
        let mut res: u32 = 0;
        init();
        enumerate_devices(&mut res as *mut u32);
        println!("{}", res);
    }
    thread::spawn(move || {
        let mut cnt = 0;
        let start = std::time::Instant::now();
        while !terminate.load(atomic::Ordering::Relaxed) {
            unsafe {
                get_frame(0);
            }
            cnt += 1;
            if cnt % 100 == 0 {
                let elapsed = start.elapsed();
                println!("fps: {}", cnt as f64 / elapsed.as_secs_f64());
            }
        }
        unsafe {
            final_();
        }
        Ok(())
    })
}
