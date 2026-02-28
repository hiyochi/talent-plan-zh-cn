use crate::common::{GetResponse, RemoveResponse, Request, SetResponse};
use crate::{KvsEngine, Result};
use log::{debug, error};
use serde_json::Deserializer;
use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

/// 键值存储的服务器。
pub struct KvsServer<E: KvsEngine> {
    engine: E,
}

impl<E: KvsEngine> KvsServer<E> {
    /// 使用给定的存储引擎创建一个 `KvsServer`。
    pub fn new(engine: E) -> Self {
        KvsServer { engine }
    }

    /// 启动服务器并监听指定地址。
    pub fn run<A: ToSocketAddrs>(mut self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = self.serve(stream) {
                        error!("处理客户端时出错: {}", e);
                    }
                }
                Err(e) => error!("连接失败: {}", e),
            }
        }
        Ok(())
    }

    fn serve(&mut self, tcp: TcpStream) -> Result<()> {
        let peer_addr = tcp.peer_addr()?;
        let reader = BufReader::new(&tcp);
        let mut writer = BufWriter::new(&tcp);
        let req_reader = Deserializer::from_reader(reader).into_iter::<Request>();

        macro_rules! send_resp {
            ($resp:expr) => {{
                let resp = $resp;
                serde_json::to_writer(&mut writer, &resp)?;
                writer.flush()?;
                debug!("已向 {} 发送响应: {:?}", peer_addr, resp);
            };};
        }

        for req in req_reader {
            let req = req?;
            debug!("从 {} 接收到请求: {:?}", peer_addr, req);
            match req {
                Request::Get { key } => send_resp!(match self.engine.get(key) {
                    Ok(value) => GetResponse::Ok(value),
                    Err(e) => GetResponse::Err(format!("{}", e)),
                }),
                Request::Set { key, value } => send_resp!(match self.engine.set(key, value) {
                    Ok(_) => SetResponse::Ok(()),
                    Err(e) => SetResponse::Err(format!("{}", e)),
                }),
                Request::Remove { key } => send_resp!(match self.engine.remove(key) {
                    Ok(_) => RemoveResponse::Ok(()),
                    Err(e) => RemoveResponse::Err(format!("{}", e)),
                }),
            };
        }
        Ok(())
    }
}