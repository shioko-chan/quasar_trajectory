use std::{collections::VecDeque, sync::mpsc};
use thiserror::Error;

/// 表示管道可能的错误类型
#[derive(Debug, Error, PartialEq)]
pub enum TubeError {
    /// 管道已经关闭，无法发送或接收数据
    #[error("This tube has shut down.")]
    TubeShutdown,
}

/// 管道的发送端结构
/// 管理数据的发送缓冲区和发送逻辑
pub struct TubeSend<T> {
    /// 内部缓冲队列，存储待发送的数据
    dishes: VecDeque<Box<T>>,
    /// 用于发送数据的通道
    supply: mpsc::Sender<Box<T>>,
    /// 用于回收缓冲区的通道
    recycle: mpsc::Receiver<Box<T>>,
}

/// 管道的接收端结构
/// 负责接收数据并支持回收空闲缓冲区
pub struct TubeRecv<T> {
    /// 用于接收数据的通道
    fetch: mpsc::Receiver<Box<T>>,
    /// 用于回收缓冲区的发送端
    refund: mpsc::Sender<Box<T>>,
}

/// 当管道结构出现逻辑错误时的 panic 信息
const TUBE_PANIC: &str = "[Tube]: data structure collapsed";

/// 创建一个管道（Tube），包括发送端和接收端
///
/// # 泛型参数
/// - `T`: 数据类型，需要实现 `Default` trait，用于初始化缓冲区
///
/// # 返回值
/// 返回一个 `(TubeSend<T>, TubeRecv<T>)` 元组，分别表示发送端和接收端
pub fn tube<T>() -> (TubeSend<T>, TubeRecv<T>)
where
    T: Default,
{
    // 创建用于数据传输的通道
    let (supply, fetch) = mpsc::channel();
    // 创建用于缓冲区回收的通道
    let (refund, recycle) = mpsc::channel();

    // 初始化缓冲区，包含两个默认的数据项
    let mut dishes = VecDeque::new();
    dishes.push_back(Box::new(Default::default()));
    dishes.push_back(Box::new(Default::default()));

    // 构造发送端和接收端
    let tube_send = TubeSend {
        dishes,
        supply,
        recycle,
    };
    let tube_recv = TubeRecv { fetch, refund };

    (tube_send, tube_recv)
}

impl<T> TubeSend<T> {
    /// 获取发送缓冲区的第一个元素的可变引用
    ///
    /// # 返回值
    /// 返回缓冲区队列中的第一个元素。
    /// 如果队列为空，将触发 panic。
    pub fn get_send_buffer(&mut self) -> &mut Box<T> {
        self.dishes.front_mut().expect(TUBE_PANIC)
    }

    /// 将缓冲区中的数据发送到接收端
    ///
    /// # 返回值
    /// - `Ok(())` 表示成功发送。
    /// - `Err(TubeError::TubeShutdown)` 表示管道已关闭。
    pub fn send(&mut self) -> Result<(), TubeError> {
        // 如果缓冲区只剩一个数据，尝试从回收通道中获取空缓冲区
        if self.dishes.len() == 1 {
            match self.recycle.try_recv() {
                Ok(empty_dish) => self.dishes.push_back(empty_dish),
                Err(mpsc::TryRecvError::Empty) => return Ok(()), // 无可用缓冲区
                Err(mpsc::TryRecvError::Disconnected) => return Err(TubeError::TubeShutdown),
            }
        }
        // 发送缓冲区中的第一个数据
        self.supply
            .send(self.dishes.pop_front().expect(TUBE_PANIC))
            .map_err(|_| TubeError::TubeShutdown)
    }
}

impl<T> TubeRecv<T> {
    /// 从接收通道中获取数据
    ///
    /// # 返回值
    /// - `Ok(Box<T>)` 表示成功接收到数据。
    /// - `Err(TubeError::TubeShutdown)` 表示发送端已关闭。
    pub fn recv(&self) -> Result<Box<T>, TubeError> {
        self.fetch.recv().map_err(|_| TubeError::TubeShutdown)
    }

    /// 回收空缓冲区，将其返还给发送端
    ///
    /// # 参数
    /// - `empty_dish`: 要回收的空缓冲区。
    ///
    /// # 返回值
    /// - `Ok(())` 表示成功回收。
    /// - `Err(TubeError::TubeShutdown)` 表示发送端已关闭。
    pub fn recycle(&self, empty_dish: Box<T>) -> Result<(), TubeError> {
        self.refund
            .send(empty_dish)
            .map_err(|_| TubeError::TubeShutdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug, PartialEq)]
    struct TestData {
        value: i32,
    }

    #[test]
    fn test_tube_send_recv() {
        let (mut tube_send, tube_recv) = tube::<TestData>();

        // Get the send buffer and modify it
        let send_buffer = tube_send.get_send_buffer();
        send_buffer.value = 42;

        // Send the buffer
        assert!(tube_send.send().is_ok());

        // Receive the buffer
        let recv_buffer = tube_recv.recv().unwrap();
        assert_eq!(recv_buffer.value, 42);
    }

    #[test]
    fn test_tube_recycle() {
        let (mut tube_send, tube_recv) = tube::<TestData>();

        // Get the send buffer and modify it
        let send_buffer = tube_send.get_send_buffer();
        send_buffer.value = 42;

        // Send the buffer
        assert!(tube_send.send().is_ok());

        // Receive the buffer
        let recv_buffer = tube_recv.recv().unwrap();
        assert_eq!(recv_buffer.value, 42);

        // Recycle the buffer
        assert!(tube_recv.recycle(recv_buffer).is_ok());

        // Get the send buffer again and check if it's recycled
        let send_buffer = tube_send.get_send_buffer();
        assert_eq!(send_buffer.value, 0); // Default value
    }

    #[test]
    fn test_tube_shutdown() {
        let (mut tube_send, tube_recv) = tube::<TestData>();

        // Drop the receiver to simulate shutdown
        drop(tube_recv);

        // Try to send a buffer
        assert_eq!(tube_send.send(), Err(TubeError::TubeShutdown));
    }

    #[test]
    fn test_tube_recv_shutdown() {
        let (tube_send, tube_recv) = tube::<TestData>();

        // Drop the sender to simulate shutdown
        drop(tube_send);

        // Try to receive a buffer
        assert_eq!(tube_recv.recv(), Err(TubeError::TubeShutdown));
    }

    #[test]
    fn test_tube_recycle_shutdown() {
        let (tube_send, tube_recv) = tube::<TestData>();

        // Drop the sender to simulate shutdown
        drop(tube_send);

        // Try to recycle a buffer
        let empty_dish = Box::new(TestData::default());
        assert_eq!(tube_recv.recycle(empty_dish), Err(TubeError::TubeShutdown));
    }
}
