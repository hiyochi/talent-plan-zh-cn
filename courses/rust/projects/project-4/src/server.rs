use crate::common::{GetResponse, RemoveResponse, Request, SetResponse};
use crate::thread_pool::ThreadPool;
use crate::{KvsEngine, Result};
use log::{debug, error};
use serde_json::Deserializer;
use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

/// 键值存储的服务器。
pub struct KvsServer<E: KvsEngine, P: ThreadPool> {
    engine: E,
    pool: P,
}

impl<E: KvsEngine, P: ThreadPool> KvsServer<E, P> {
    /// 使用指定的存储引擎创建一个 `KvsServer`。
    pub fn new(engine: E, pool: P) -> Self {
        KvsServer { engine, pool }
    }

    /// 启动服务器，在指定地址上监听。
    pub fn run<A: ToSocketAddrs>(self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            let engine = self.engine.clone();
            self.pool.spawn(move || match stream {
                Ok(stream) => {
                    if let Err(e) = serve(engine, stream) {
                        error!("处理客户端时出错: {}", e);
                    }
                }
                Err(e) => error!("连接失败: {}", e),
            })
        }
        Ok(())
    }
}

fn serve<E: KvsEngine>(engine: E, tcp: TcpStream) -> Result<()> {
    let peer_addr = tcp.peer_addr()?;
    let reader = BufReader::new(&tcp);
    let mut writer = BufWriter::new(&tcp);
    let req_reader = Deserializer::from_reader(reader).into_iter::<Request>();

    macro_rules! send_resp {
        ($resp:expr) => {{
            let resp = $resp;
            serde_json::to_writer(&mut writer, &resp)?;
            writer.flush()?;
            debug!("已向 {}: {:?} 发送响应", peer_addr, resp);
        };};
    }

    for req in req_reader {
        let req = req?;
        debug!("从 {}: {:?} 接收到请求", peer_addr, req);
        match req {
            Request::Get { key } => send_resp!(match engine.get(key) {
                Ok(value) => GetResponse::Ok(value),
                Err(e) => GetResponse::Err(format!("{}", e)),
            }),
            Request::Set { key, value } => send_resp!(match engine.set(key, value) {
                Ok(_) => SetResponse::Ok(()),
                Err(e) => SetResponse::Err(format!("{}", e)),
            }),
            Request::Remove { key } => send_resp!(match engine.remove(key) {
                Ok(_) => RemoveResponse::Ok(()),
                Err(e) => RemoveResponse::Err(format!("{}", e)),
            }),
        };
    }
    Ok(())
}