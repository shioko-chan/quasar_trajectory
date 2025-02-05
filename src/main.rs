use std::{
    sync::{atomic::AtomicBool, Arc},
    thread::{self, JoinHandle},
};

fn main() {
    config::load_config();
    let stop_signal = Arc::new(AtomicBool::new(false));

    ctrlc::set_handler({
        let stop_signal = stop_signal.clone();
        move || {
            stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    })
    .expect("Error setting Ctrl-C handler");
    let handle = detector::detector(stop_signal.clone());
    handle.join().iter().all(|res| res.is_ok());
}
