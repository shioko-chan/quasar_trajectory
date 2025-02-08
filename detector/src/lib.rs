#[cfg(feature = "hikvision")]
include!(concat!(env!("OUT_DIR"), "/hikcamera/camera.rs"));

#[cfg(feature = "mindvision")]
include!(concat!(env!("OUT_DIR"), "hikcamera/camera.rs"));

use anyhow::{anyhow, bail, ensure};
use config::CONFIG;
use log::info;
use messages::{new_tube, TubeRecv, TubeSend};

use opencv::{self as cv, core::*};
use rand::{rngs::ThreadRng, Rng};
// use tungstenite::WebSocket;

use std::{
    ffi::CString,
    io,
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
    thread::{self, JoinHandle, Thread},
    time::Instant,
};

// #[cfg(feature = "gui")]
// mod gui {
//     use super::*;
//     fn wait_connection(terminate: Arc<AtomicBool>) -> Result<TcpStream, anyhow::Error> {
//         let server = TcpListener::bind("0.0.0.0:16700").expect("[可视化] 无法绑定端口 16700");
//         server
//             .set_nonblocking(true)
//             .expect("[可视化] 无法设置TCP Server为非阻塞");
//         let (stream, socket) = loop {
//             ensure!(!terminate.load(atomic::Ordering::Relaxed), "线程被中断");
//             let res = server.accept();
//             if res.is_ok() {
//                 break res;
//             }
//             ensure!(
//                 matches!(res, Err(e) if e.kind()!=io::ErrorKind::WouldBlock),
//                 "无法接受连接"
//             );
//         }?;
//         info!("[可视化] 相机可视化已连接到: {:?}", socket);
//         Ok(stream)
//     }

//     pub struct ImageWs {
//         ws: WebSocket<TcpStream>,
//         rng: ThreadRng,
//         cnt: u8,
//         fps: f32,
//         start: Instant,
//         buffer: Vector<u8>,
//     }

//     impl ImageWs {
//         pub fn new(terminate: Arc<AtomicBool>) -> Result<Self, anyhow::Error> {
//             let stream = wait_connection(terminate.clone())?;
//             return Ok(Self {
//                 ws: tungstenite::accept(stream).expect("[可视化] WebSocket 握手失败"),
//                 rng: rand::rng(),
//                 cnt: 0,
//                 fps: 0.0,
//                 start: Instant::now(),
//                 buffer: Vector::new(),
//             });
//         }
//     }
// }

fn stringify_err(ret: APIError) -> String {
    format!(
        "错误来源：{}， 错误码：{:X}",
        if ret.is_thirdparty_err == 1 {
            "`第三方相机SDK`"
        } else {
            "`相机API`"
        },
        ret.code as usize
    )
}

fn init_sdk() -> Result<(), anyhow::Error> {
    let ret;
    unsafe {
        ret = init();
    }
    ensure!(ret.code == 0, "相机SDK初始化失败；{}", stringify_err(ret));
    Ok(())
}

fn init_cameras() -> Result<(), anyhow::Error> {
    let mut res: u32 = 0;
    let ret;
    unsafe {
        ret = enumerate_devices(&mut res as *mut u32);
    }
    ensure!(ret.code == 0, "枚举设备失败；{}", stringify_err(ret));
    ensure!(res != 0, "未发现设备");

    info!("发现相机数量: {}", res);
    Ok(())
}

fn camera_launch(
    sender: TubeSend<Mat>,
    terminate: Arc<AtomicBool>,
) -> JoinHandle<Result<(), anyhow::Error>> {
    thread::spawn(move || {
        init_sdk()?;
        init_cameras()?;

        const CAM_ID: u32 = 0;
        let config::Camera {
            exposure_time,
            exposure_auto,
            gain,
            gain_auto,
            width,
            height,
            ..
        } = CONFIG
            .get()
            .expect("锁中毒")
            .as_ref()
            .expect("CONFIG未初始化")
            .camera;
        let mut ret;
        #[cfg(feature = "hikvision")]
        {
            unsafe {
                // ExposureAuto(自动曝光) 枚举值：0: 自动曝光关闭，1: 自动曝光一次 2: 自动曝光连续。参见海康相机用户手册
                ret = set_enum_param(
                    CAM_ID,
                    CString::new("ExposureAuto").unwrap().as_ptr(),
                    if exposure_auto { 2 } else { 0 },
                );
            }
            ensure!(ret.code == 0, "设置曝光模式失败；{}", stringify_err(ret));
            if !exposure_auto {
                unsafe {
                    // ExposureTime(曝光时间) 单位：微秒
                    ret = set_float_param(
                        CAM_ID,
                        CString::new("ExposureTime").unwrap().as_ptr(),
                        exposure_time,
                    );
                }
                ensure!(ret.code == 0, "设置曝光时间失败；{}", stringify_err(ret));
            }

            unsafe {
                // GainAuto(自动增益) 枚举值：0: 自动增益关闭，1: 自动增益一次 2: 自动增益连续。参见海康相机用户手册
                ret = set_enum_param(
                    CAM_ID,
                    CString::new("GainAuto").unwrap().as_ptr(),
                    if gain_auto { 2 } else { 0 },
                );
            }
            ensure!(ret.code == 0, "设置增益模式失败；{}", stringify_err(ret));

            if !gain_auto {
                unsafe {
                    // Gain(增益) 单位：分贝
                    ret = set_float_param(CAM_ID, CString::new("Gain").unwrap().as_ptr(), gain);
                }
                ensure!(ret.code == 0, "设置增益失败；{}", stringify_err(ret));
            }

            unsafe {
                // Width(图像宽度) 单位：像素
                ret = set_int_param(CAM_ID, CString::new("Width").unwrap().as_ptr(), width);
            }
            ensure!(ret.code == 0, "设置图像宽度失败；{}", stringify_err(ret));

            unsafe {
                // Height(图像高度) 单位：像素
                ret = set_int_param(CAM_ID, CString::new("Height").unwrap().as_ptr(), height);
            }
            ensure!(ret.code == 0, "设置图像高度失败；{}", stringify_err(ret));
        }
        #[cfg(feature = "gui")]
        {
            gui::ImageWs::new(terminate.clone())?;
            let size = (width * height * 3) as u32;

            let (mut rng, mut cnt, mut fps, mut start) = (rand::rng(), 0, 0.0, Instant::now());

            let mut mat = unsafe {
                Mat::new_rows_cols(height as i32, width as i32, CV_8UC3)
                    .expect("[可视化] 无法创建Mat")
            };
            let server = TcpListener::bind("0.0.0.0:16700").expect("[可视化] 无法绑定端口 16700");
            server
                .set_nonblocking(true)
                .expect("[可视化] 无法设置TCP Server为非阻塞");
            while !terminate.load(atomic::Ordering::Relaxed) {
                if let Ok((stream, socket)) = server.accept() {
                    info!("[可视化] 相机可视化已连接到: {:?}", socket);
                    let mut websocket =
                        tungstenite::accept(stream).expect("[可视化] WebSocket 握手失败");
                    while !terminate.load(atomic::Ordering::Relaxed) {
                        unsafe {
                            ret = get_frame(CAM_ID, mat.data_mut(), size);
                        }
                        if ret.code != 0 {
                            continue;
                        }
                        cnt += 1;
                        if cnt == 100 {
                            fps = 100.0 / start.elapsed().as_secs_f32();
                            info!("fps: {fps}");
                            start = Instant::now();
                            cnt = 0;
                        }
                        let probability = 30.0 / fps;
                        if rng.random::<f32>() < probability {
                            let mut buf = Vector::new();
                            cv::imgcodecs::imencode(".jpg", &mat, &mut buf, &Vector::new())
                                .expect("[可视化] 无法编码图像");
                            websocket
                                .write(tungstenite::Message::Binary(buf.to_vec().into()))
                                .expect("[可视化] Websocket 发送图片失败");
                        }
                    }
                }
            }
        }
        #[cfg(not(feature = "gui"))]
        {
            while !terminate.load(atomic::Ordering::Relaxed) {
                unsafe {
                    // get_frame();
                }
            }
        }
        unsafe {
            final_();
        }
        info!("已关闭相机线程");
        Ok(())
    })
}

pub fn detector(terminate: Arc<AtomicBool>) -> JoinHandle<Result<(), anyhow::Error>> {
    let (tx, rx) = new_tube();
    camera_launch(tx, terminate.clone())
}
