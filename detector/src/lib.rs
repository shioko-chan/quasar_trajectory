//  ____   ____ ____     __     __
// |  _ \ / ___/ ___|    \ \   / /
// | |_) | |   \___ \ ____\ \ / /
// |  _ <| |___ ___) |_____\ V /
// |_| \_\\____|____/       \_/
//
/*
DeviceScanType
DeviceModel
*/
//! # 工业相机参数说明
//!
//! 本文档介绍了工业相机常见参数的功能、数据类型、取值范围以及读写权限等信息。参数按功能模块分组，主要包括采集控制、模拟控制、LUT 控制、编码器控制、频率转换器控制以及阴影校正。
//!
//! ## 1. Acquisition Control（采集控制）
//!
//! - **AcquisitionMode (IEnumeration)**  
//!   - **取值**：0: SingleFrame, 1: MultiFrame, 2: Continuous  
//!   - **说明**：设置采集模式；单帧模式下采集一帧后停止，多帧模式下每次触发采集固定帧数，连续模式下相机持续采集图像。 (*R/（W）*)
//!
//! - **AcquisitionStart (ICommand)**  
//!   - **说明**：启动采集命令。（写）
//!
//! - **AcquisitionStop (ICommand)**  
//!   - **说明**：结束采集命令。（写）
//!
//! - **AcquisitionBurstFrameCount (IInteger)**  
//!   - **要求**：≥ 0  
//!   - **说明**：触发采集时一次获取的帧数。（读/写）
//!
//! - **AcquisitionLineRate (IInteger)**  
//!   - **要求**：≥ 1  
//!   - **说明**：设置扫描线速率。（读/写）
//!
//! - **AcquisitionLineRateEnable (IBoolean)**  
//!   - **取值**：True/False  
//!   - **说明**：是否启用行速率控制。（读/写）
//!
//! - **ResultingLineRate (IInteger)**  
//!   - **说明**：实际采集行速率（Hz），只读。
//!
//! - **ResultingFrameRate (IFloat)**  
//!   - **说明**：实际采集帧率（fps），只读。
//!
//! - **TriggerSelector (IEnumeration)**  
//!   - **取值**：  
//!     0: AcquisitionStart, 1: AcquisitionEnd, 2: AcquisitionActive,  
//!     3: FrameStart, 4: FrameEnd, 5: FrameActive,  
//!     6: FrameBurstStart, 7: FrameBurstEnd, 8: FrameBurstActive,  
//!     9: LineStart, 10: ExposureStart, 11: ExposureEnd, 12: ExposureActive  
//!   - **说明**：选择触发事件类型，用于确定触发采集的时刻。（读/写）
//!
//! - **TriggerMode (IEnumeration)**  
//!   - **取值**：0: Off, 1: On  
//!   - **说明**：是否启用外部触发模式。（读/写）
//!
//! - **TriggerSource (IEnumeration)**  
//!   - **取值**：0: Line0, 1: Line1, 2: Line2, 3: Line3, 4: Counter0, 7: Software, 8: FrequencyConverter  
//!   - **说明**：选择触发信号来源。（读/写）
//!
//! - **TriggerActivation (IEnumeration)**  
//!   - **取值**：0: RisingEdge, 1: FallingEdge, 2: LevelHigh, 3: LevelLow  
//!   - **说明**：触发激活方式。（读/写）
//!
//! - **TriggerDelay (IFloat)**  
//!   - **要求**：≥ 0.0，单位：微秒 (us)  
//!   - **说明**：触发信号到实际采集开始的延迟。（读/写）
//!
//! - **ExposureMode (IEnumeration)**  
//!   - **取值**：0: Timed, 1: TriggerWidth  
//!   - **说明**：曝光模式；Timed 模式下由 ExposureTime 控制，TriggerWidth 模式下与触发信号持续时间一致。（读/写）
//!
//! - **ExposureTime (IFloat)**  
//!   - **要求**：≥ 0.0，单位：微秒  
//!   - **说明**：曝光时间。（读/写）
//!
//! - **ExposureAuto (IEnumeration)**  
//!   - **取值**：0: Off, 1: Once, 2: Continuous  
//!   - **说明**：自动曝光模式选择。（读/写）
//!
//! - **AutoExposureTimeLowerLimit (IInteger)** & **AutoExposureTimeupperLimit (IInteger)**  
//!   - **要求**：≥ 0.0，单位：微秒  
//!   - **说明**：自动曝光的时间下限和上限。（读/写或只读，根据 API 定义）
//!
//! - **FrameTimeoutEnable (IBoolean)**  
//!   - **取值**：0: Off, 1: On  
//!   - **说明**：是否启用帧超时机制。（读/写）
//!
//! - **FrameTimeoutTime (IInteger)**  
//!   - **要求**：≥ 87，单位：毫秒  
//!   - **说明**：帧超时周期。（读/写）
//!
//! ## 2. Analog Control（模拟控制）
//!
//! - **Gain (IFloat)**  
//!   - **要求**：≥ 0.0，单位：dB  
//!   - **说明**：模拟增益值，影响图像亮度与噪点。（读/写）
//!
//! - **GainAuto (IEnumeration)**  
//!   - **取值**：0: Off, 1: Once, 2: Continuous  
//!   - **说明**：自动增益控制模式。（读/写）
//!
//! - **AutoGainLowerLimit (IFloat)** & **AutoGainupperLimit (IFloat)**  
//!   - **要求**：≥ 0.0，单位：dB  
//!   - **说明**：自动增益调节的下限与上限。（读/写）
//!
//! - **ADCGainEnable (IBoolean)**  
//!   - **取值**：0: Off, 1: On  
//!   - **说明**：是否启用 ADC 增益。（读/写）
//!
//! - **DigitalShift (IFloat)**  
//!   - **要求**：≥ 0.0  
//!   - **说明**：数字偏移调整，通常用于修正数字化过程中产生的固定偏差。（只读）
//!
//! - **DigitalShiftEnable (IBoolean)**  
//!   - **取值**：0: Off, 1: On  
//!   - **说明**：是否启用数字偏移功能。（读/写）
//!
//! - **Brightness (IInteger)**  
//!   - **要求**：≥ 0  
//!   - **说明**：图像亮度设置，通常范围为 0～255。（读/写）
//!
//! - **BlackLevel (IFloat)**  
//!   - **要求**：≥ 0.0  
//!   - **说明**：黑电平校正，用于调整无光时的输出灰度。（读/写）
//!
//! - **BlackLevelEnable (IBoolean)**  
//!   - **取值**：0: Off, 1: On  
//!   - **说明**：是否启用黑电平校正。（读/写）
//!
//! - **BalanceWhiteAuto (IEnumeration)**  
//!   - **取值**：0: Off, 1: Continuous, 2: Once  
//!   - **说明**：自动白平衡模式设置。（读/写）
//!
//! - **BalanceRatioSelector (IEnumeration)**  
//!   - **取值**：0: Red, 1: Green, 2: Blue  
//!   - **说明**：白平衡比率选择器（只读）。
//!
//! - **BalanceRatio (IInteger)**  
//!   - **要求**：≥ 0  
//!   - **说明**：白平衡值显示（只读）。
//!
//! - **Gamma (IFloat)**  
//!   - **要求**：> 0.0  
//!   - **说明**：Gamma 校正值，用于调整非线性亮度响应。（读/写）
//!
//! - **GammaSelector (IEnumeration)**  
//!   - **取值**：1: User, 2: sRGB  
//!   - **说明**：选择 Gamma 曲线类型。（读/写）
//!
//! - **GammaEnable (IBoolean)**  
//!   - **取值**：0: Off, 1: On  
//!   - **说明**：是否启用 Gamma 校正。（读/写）
//!
//! - **Hue (IInteger)**  
//!   - **要求**：≥ 0  
//!   - **说明**：色相调整（只读）。
//!
//! - **HueEnable (IBoolean)**  
//!   - **取值**：0: Off, 1: On  
//!   - **说明**：是否启用色相调整。（读/写）
//!
//! - **Saturation (IInteger)**  
//!   - **要求**：≥ 0  
//!   - **说明**：饱和度调整（只读）。
//!
//! - **SaturationEnable (IBoolean)**  
//!   - **取值**：0: Off, 1: On  
//!   - **说明**：是否启用饱和度调整。（读/写）
//!
//! - **AutoFunctionAOISelector (IEnumeration)**  
//!   - **取值**：0: AOI1, 1: AOI2  
//!   - **说明**：自动功能参考区域选择。（读/写）
//!
//! - **AutoFunctionAOIWidth (IInteger)**, **AutoFunctionAOIHeight (IInteger)**  
//!   - **要求**：≥ 0  
//!   - **说明**：自动功能区域的宽度与高度。（读/写）
//!
//! - **AutoFunctionAOIOffsetX (IInteger)**  
//!   - **要求**：≥ 0  
//!   - **说明**：自动功能区域水平偏移（只读）。
//!
//! - **AutoFunctionAOIUsageIntensity (IBoolean)**  
//!   - **取值**：0: Off, 1: On  
//!   - **说明**：是否根据 AOI 区域强度自动调整曝光。（读/写）
//!
//! - **AutoFunctionAOIUsageWhiteBalance (IBoolean)**  
//!   - **取值**：0: Off, 1: On  
//!   - **说明**：是否根据 AOI 区域自动调整白平衡（只读）。
//!
//! ## 3. LUT Control（查找表控制）
//!
//! - **LUTSelector (IEnumeration)**  
//!   - **取值**：0: Luminance, 1: Red, 2: Green, 3: Blue  
//!   - **说明**：选择应用 LUT 的通道（亮度或各颜色通道），用于调整图像输出的亮度及色彩。（读/写）
//!
//! - **LUTEnable (IBoolean)**  
//!   - **取值**：True/False  
//!   - **说明**：启用或禁用 LUT 功能。（读/写）
//!
//! - **LUTIndex (IInteger)**  
//!   - **要求**：≥ 0  
//!   - **说明**：指定查找表中要调整的索引位置。（读/写）
//!
//! - **LUTValue (IInteger)**  
//!   - **说明**：设置 LUT 对应索引处的输出值；具体数值范围由设备定义。（读/写）
//!
//! ## 4. Encoder Control（编码器控制）
//!
//! - **EncoderSelector (IEnumeration)**  
//!   - **取值**：0: Encoder0, 1: Encoder1, 2: Encoder2  
//!   - **说明**：选择使用哪一个编码器进行数据采集。（读/写）
//!
//! - **EncoderSourceA / EncoderSourceB (IEnumeration)**  
//!   - **取值**：0: Line0, 1: Line1, 2: Line2, 3: Line3  
//!   - **说明**：分别选择编码器 A 和 B 的信号来源。（读/写）
//!
//! - **EncoderTriggerMode (IEnumeration)**  
//!   - **取值**：0: AnyDirection, 1: ForwardOnly  
//!   - **说明**：编码器触发模式，决定是否响应任意方向的运动或仅正向。（读/写）
//!
//! - **EncoderCounterMode (IEnumeration)**  
//!   - **取值**：0: IgnoreDirection, 1: FollowDirection  
//!   - **说明**：编码器计数模式，是否考虑运动方向。（读/写）
//!
//! - **EncoderCounter (IInteger)**  
//!   - **说明**：当前编码器计数值。（只读）
//!
//! - **EncoderCounterMax (IInteger)**  
//!   - **要求**：≥ 1  
//!   - **说明**：编码器计数器最大值。（读/写）
//!
//! - **EncoderCounterReset (ICommand)**  
//!   - **说明**：复位编码器计数器。（读/写）
//!
//! - **EncoderMaxReverseCounter (IInteger)**  
//!   - **要求**：≥ 1  
//!   - **说明**：编码器反向计数器最大值。（读/写）
//!
//! - **EncoderReverseCounterReset (ICommand)**  
//!   - **说明**：复位编码器反向计数器。（读/写）
//!
//! ## 5. Frequency Converter Control（频率转换器控制）
//!
//! - **InputSource (IEnumeration)**  
//!   - **取值**：0: Line0, 1: Line1, 2: Line2, 3: Line3  
//!   - **说明**：选择输入信号来源，用于频率转换。（读/写）
//!
//! - **SignalAlignment (IEnumeration)**  
//!   - **取值**：0: RisingEdge, 1: FallingEdge  
//!   - **说明**：设置信号沿对齐方式。（读/写）
//!
//! - **PreDivider (IInteger)**  
//!   - **要求**：≥ 1  
//!   - **说明**：预分频器设置，用于初步降低输入信号频率。（读/写）
//!
//! - **Multiplier (IInteger)**  
//!   - **要求**：≥ 1  
//!   - **说明**：乘法器设置，用于放大预分频后的信号。（读/写）
//!
//! - **PostDivider (IInteger)**  
//!   - **要求**：≥ 1  
//!   - **说明**：后分频器设置，用于进一步调整输出频率。（读/写）
//!
//! ## 6. Shading Correction（阴影校正）
//!
//! - **ShadingSelector (IEnumeration)**  
//!   - **取值**：0: FPNCCorrection, 1: PRNUCCorrection  
//!   - **说明**：选择阴影校正模式，分别对应固定图案噪声校正和光响应非均匀性校正。（读/写）
//!
//! - **ActivateShading (ICommand)**  
//!   - **说明**：激活阴影校正。（读/写）
//!
//! - **NUCEnable (IBoolean)**  
//!   - **取值**：0: Off, 1: On  
//!   - **说明**：启用非均匀性校正。（读/写）
//!
//! - **PRNUCEnable (IBoolean)**  
//!   - **取值**：0: Off, 1: On  
//!   - **说明**：启用 PRNU 校正。（读/写）
//!
//! ## 额外说明
//!
//! 工业相机提供大量参数以满足不同应用场景的需求。为实现运行时的动态参数调整，可通过相机 API 的读取和写入接口对各参数进行实时查询和更新。
//! 建议构建一个统一的参数管理层，将各参数封装为统一的数据结构，并利用线程安全的共享状态（例如 `Arc<RwLock<...>>`），再结合
//! UI 框架动态展示参数状态，从而实现参数的实时监控和调整。
//!
//! 各参数的具体取值范围、默认值以及支持情况可能因相机型号和固件版本有所不同，详细信息请参考相应的产品手册或厂商文档。

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

use anyhow::{anyhow, bail, ensure};
use config::CONFIG;
use log::info;
use utility::{new_tube, TubeRecv, TubeSend};

use opencv::{self as cv, core::*};
use rand::{rngs::ThreadRng, Rng};

// use tungstenite::WebSocket;

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

fn camera_launch(sender: TubeSend<Mat>) -> JoinHandle<Result<(), anyhow::Error>> {
    thread::spawn(move || {
        init_sdk()?;
        init_cameras()?;
        config_camera(0)?;

        // #[cfg(feature = "gui")]
        // {
        //     gui::ImageWs::new(terminate.clone())?;
        //     let size = (width * height * 3) as u32;

        //     let (mut rng, mut cnt, mut fps, mut start) = (rand::rng(), 0, 0.0, Instant::now());

        //     let mut mat = unsafe {
        //         Mat::new_rows_cols(height as i32, width as i32, CV_8UC3)
        //             .expect("[可视化] 无法创建Mat")
        //     };
        //     let server = TcpListener::bind("0.0.0.0:16700").expect("[可视化] 无法绑定端口 16700");
        //     server
        //         .set_nonblocking(true)
        //         .expect("[可视化] 无法设置TCP Server为非阻塞");
        //     while !terminate.load(atomic::Ordering::Relaxed) {
        //         if let Ok((stream, socket)) = server.accept() {
        //             info!("[可视化] 相机可视化已连接到: {:?}", socket);
        //             let mut websocket =
        //                 tungstenite::accept(stream).expect("[可视化] WebSocket 握手失败");
        //             while !terminate.load(atomic::Ordering::Relaxed) {
        //                 unsafe {
        //                     ret = get_frame(CAM_ID, mat.data_mut(), size);
        //                 }
        //                 if ret.code != 0 {
        //                     continue;
        //                 }
        //                 cnt += 1;
        //                 if cnt == 100 {
        //                     fps = 100.0 / start.elapsed().as_secs_f32();
        //                     info!("fps: {fps}");
        //                     start = Instant::now();
        //                     cnt = 0;
        //                 }
        //                 let probability = 30.0 / fps;
        //                 if rng.random::<f32>() < probability {
        //                     let mut buf = Vector::new();
        //                     cv::imgcodecs::imencode(".jpg", &mat, &mut buf, &Vector::new())
        //                         .expect("[可视化] 无法编码图像");
        //                     websocket
        //                         .write(tungstenite::Message::Binary(buf.to_vec().into()))
        //                         .expect("[可视化] Websocket 发送图片失败");
        //                 }
        //             }
        //         }
        //     }
        // }
        #[cfg(not(feature = "gui"))]
        {
            while !terminate.load(atomic::Ordering::Relaxed) {
                unsafe {
                    // get_frame();
                }
            }
        }
    })
}

pub fn detector() -> JoinHandle<Result<(), anyhow::Error>> {
    let (tx, rx) = new_tube();
    camera_launch(tx)
}
