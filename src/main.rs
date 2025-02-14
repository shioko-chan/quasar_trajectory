use std::{
    sync::{atomic::AtomicBool, Arc},
    thread::{self, JoinHandle},
};

use log::{debug, error, info, warn};

use utility::stop_all;

fn main() {
    env_logger::init();

    ctrlc::set_handler({
        move || {
            stop_all();
        }
    })
    .expect("Error setting Ctrl-C handler");
    // let handle = detector::detector();
    camera::test();
    // handle.join().unwrap().unwrap_or_else(|err| {
    //     error!("检测器异常退出: {}", err);
    // });
}
