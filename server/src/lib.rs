use config::CONFIG;
use crossbeam_channel::{select, unbounded, RecvTimeoutError};
use opencv::{self as cv, core::*};
use rand::{rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    io,
    marker::PhantomData,
    net::TcpListener,
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};
use ultraviolet::Vec3;
use utility::{ensure_or_stop, expect_or_stop, is_stopped, stop_all, unwrap_or_stop};

static SENDER: OnceLock<crossbeam_channel::Sender<tungstenite::Message>> = OnceLock::new();

fn send(msg: tungstenite::Message) -> anyhow::Result<()> {
    let sender = SENDER.get_or_init(|| {
        let (sender, receiver) = unbounded();
        launch_server(receiver);
        sender
    });
    Ok(sender.send(msg)?)
}

fn launch_server(receiver: crossbeam_channel::Receiver<tungstenite::Message>) {
    let server = unwrap_or_stop!(TcpListener::bind("0.0.0.0:25801"));
    server
        .set_nonblocking(true)
        .expect("[可视化][ERR-01] 无法设置TCP Server为非阻塞");

    thread::spawn(move || {
        for res in server.incoming() {
            if matches!(res, Err(ref e) if e.kind() == io::ErrorKind::WouldBlock) {
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            let stream = expect_or_stop!(res, "[可视化][ERR-02] 建立TCP连接时出错");
            let mut websocket = expect_or_stop!(
                tungstenite::accept(stream),
                "[可视化][ERR-03] 无法建立WebSocket连接"
            );

            while !is_stopped() {
                let res = receiver.recv_timeout(Duration::from_millis(10));
                if matches!(res, Err(RecvTimeoutError::Timeout)) {
                    continue;
                }
                let message = expect_or_stop!(res, "[可视化][ERR-04] Channel接收消息失败");

                let res = websocket.send(message);
                if matches!(res, Err(tungstenite::Error::ConnectionClosed)) {
                    break;
                }
                expect_or_stop!(res, "[可视化][ERR-05] Websocket 发送消息失败");
            }
            if is_stopped() {
                break;
            }
        }
    });
}

pub struct FPSMonitor {
    rng: ThreadRng,
    cnt: u8,
    fps: Option<f32>,
    start: Instant,
    max_fps: u8,
}

impl FPSMonitor {
    const ESTIMATE_CNT: u8 = 100;

    pub fn new(max_fps: u8) -> Self {
        Self {
            rng: rand::rng(),
            cnt: 0,
            fps: None,
            start: Instant::now(),
            max_fps,
        }
    }

    pub fn limit(&mut self) -> bool {
        self.cnt += 1;
        if self.cnt >= Self::ESTIMATE_CNT {
            self.fps = Some(Self::ESTIMATE_CNT as f32 / self.start.elapsed().as_secs_f32());
            self.cnt = 0;
            self.start = Instant::now();
        }
        if let Some(fps) = self.fps {
            let probability = self.max_fps as f32 / fps;
            self.rng.random::<f32>() < probability
        } else {
            false
        }
    }
}

#[derive(Serialize)]
struct Message<'a, T> {
    identifier: &'a String,
    data: T,
}

pub struct OnceSender {
    identifier: String,
}

impl OnceSender {
    pub fn new(identifier: String) -> Self {
        Self { identifier }
    }

    pub fn send<T>(&self, data: T) -> anyhow::Result<()>
    where
        T: Serialize,
    {
        send(tungstenite::Message::Binary(
            rmp_serde::to_vec(&Message {
                identifier: &self.identifier,
                data,
            })?
            .into(),
        ))
    }
}

pub struct PeriodicSender {
    fps_monitor: FPSMonitor,
    identifier: String,
}

impl PeriodicSender {
    pub fn new(identifier: String, max_fps: u8) -> Self {
        Self {
            fps_monitor: FPSMonitor::new(max_fps),
            identifier,
        }
    }

    pub fn send<T>(&mut self, data: T) -> anyhow::Result<()>
    where
        T: Serialize,
    {
        if !self.fps_monitor.limit() {
            return Ok(());
        }
        send(tungstenite::Message::Binary(
            rmp_serde::to_vec(&Message {
                identifier: &self.identifier,
                data,
            })?
            .into(),
        ))
    }
}

pub struct ImageSender {
    fps_monitor: FPSMonitor,
    identifier: String,
    buffer: Vector<u8>,
}

impl ImageSender {
    pub fn new(identifier: String, max_fps: u8) -> Self {
        Self {
            fps_monitor: FPSMonitor::new(max_fps),
            identifier,
            buffer: Vector::new(),
        }
    }

    pub fn send(&mut self, mat: &Mat) -> anyhow::Result<()> {
        if !self.fps_monitor.limit() {
            return Ok(());
        }
        cv::imgcodecs::imencode(".jpg", mat, &mut self.buffer, &Vector::new())?;
        send(tungstenite::Message::Binary(
            rmp_serde::to_vec(&Message {
                identifier: &self.identifier,
                data: self.buffer.as_slice(),
            })?
            .into(),
        ))
    }
}

pub struct VideoSender(PeriodicSender, ImageSender);

impl VideoSender {
    pub fn new(identifier: String, max_fps: u8, fps_report_freq: u8) -> Self {
        Self(
            PeriodicSender::new(format!("{}的帧率", identifier), fps_report_freq),
            ImageSender::new(identifier, max_fps),
        )
    }

    pub fn send(&mut self, mat: &Mat) -> anyhow::Result<()> {
        self.1.send(mat)?;
        self.0.send(self.0.fps_monitor.fps.unwrap_or(0.0))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
