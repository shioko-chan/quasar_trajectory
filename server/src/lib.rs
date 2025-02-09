use crossbeam_channel::{unbounded as channel, Receiver, Sender, TryRecvError};
use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};
use utility::{is_stopped, stop_all};

pub type BoxedMessage = Box<dyn Send>;
static SENDER: OnceLock<Sender<BoxedMessage>> = OnceLock::new();

pub fn send(msg: BoxedMessage) {
    let sender = SENDER.get_or_init(|| {
        let (sender, receiver) = channel();
        launch_server(receiver);
        sender
    });
    sender.send(msg).unwrap();
}

fn launch_server(receiver: Receiver<BoxedMessage>) {
    std::thread::spawn(|| {
        while !is_stopped() {
            //
        }
    });
}

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

//
// }

// /// WebSocket 会话结构体，保存心跳时间
// struct MyWebSocket {
//     /// 记录上次收到 ping/pong 的时间
//     hb: Instant,
// }

// impl MyWebSocket {
//     fn new() -> Self {
//         Self { hb: Instant::now() }
//     }

//     /// 启动心跳检测，间隔 5 秒发送 ping，并在 10 秒内未收到响应则关闭连接
//     fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
//         ctx.run_interval(Duration::from_secs(5), |act, ctx| {
//             if Instant::now().duration_since(act.hb) > Duration::from_secs(10) {
//                 println!("心跳超时，关闭连接");
//                 ctx.stop();
//                 return;
//             }
//             ctx.ping(b"");
//         });
//     }
// }

// impl Actor for MyWebSocket {
//     type Context = ws::WebsocketContext<Self>;

//     /// 当 Actor 启动时，启动心跳检测
//     fn started(&mut self, ctx: &mut Self::Context) {
//         self.hb(ctx);
//     }
// }

// /// 实现 StreamHandler 用于处理 WebSocket 消息
// impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
//     fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
//         match msg {
//             Ok(ws::Message::Ping(msg)) => {
//                 self.hb = Instant::now();
//                 ctx.pong(&msg);
//             }
//             Ok(ws::Message::Pong(_)) => {
//                 self.hb = Instant::now();
//             }
//             Ok(ws::Message::Text(text)) => {
//                 // 回显文本消息
//                 ctx.text(text);
//             }
//             Ok(ws::Message::Binary(bin)) => {
//                 // 回显二进制消息
//                 ctx.binary(bin);
//             }
//             Ok(ws::Message::Close(reason)) => {
//                 ctx.close(reason);
//                 ctx.stop();
//             }
//             _ => (),
//         }
//     }
// }

// /// HTTP handler，将请求升级为 WebSocket 连接
// async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
//     ws::start(MyWebSocket::new(), &req, stream)
// }

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     println!("WebSocket 服务启动于 127.0.0.1:8080");
//     HttpServer::new(|| App::new().route("/ws/", web::get().to(ws_index)))
//         .bind("127.0.0.1:8080")?
//         .run()
//         .await
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
