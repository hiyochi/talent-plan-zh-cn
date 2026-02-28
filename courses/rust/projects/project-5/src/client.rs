use crate::common::{Request, Response};
use crate::KvsError;
use std::net::SocketAddr;
use tokio::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio_serde_json::{ReadJson, WriteJson};

/// 键值存储客户端
pub struct KvsClient {
    read_json: ReadJson<FramedRead<ReadHalf<TcpStream>, LengthDelimitedCodec>, Response>,
    write_json: WriteJson<FramedWrite<WriteHalf<TcpStream>, LengthDelimitedCodec>, Request>,
}

impl KvsClient {
    /// 连接到 `addr` 以访问 `KvsServer`。
    pub fn connect(addr: SocketAddr) -> impl Future<Item = Self, Error = KvsError> {
        TcpStream::connect(&addr)
            .map(|tcp| {
                let (read_half, write_half) = tcp.split();
                let read_json =
                    ReadJson::new(FramedRead::new(read_half, LengthDelimitedCodec::new()));
                let write_json =
                    WriteJson::new(FramedWrite::new(write_half, LengthDelimitedCodec::new()));
                KvsClient {
                    read_json,
                    write_json,
                }
            })
            .map_err(|e| e.into())
    }

    /// 从服务器获取指定键的值。
    pub fn get(self, key: String) -> impl Future<Item = (Option<String>, Self), Error = KvsError> {
        self.send_request(Request::Get { key })
            .and_then(move |(resp, client)| match resp {
                Some(Response::Get(value)) => Ok((value, client)),
                Some(Response::Err(msg)) => Err(KvsError::StringError(msg)),
                Some(_) => Err(KvsError::StringError("Invalid response".to_owned())),
                None => Err(KvsError::StringError("No response received".to_owned())),
            })
    }

    /// 在服务器中设置字符串键的值。
    pub fn set(self, key: String, value: String) -> impl Future<Item = Self, Error = KvsError> {
        self.send_request(Request::Set { key, value })
            .and_then(move |(resp, client)| match resp {
                Some(Response::Set) => Ok(client),
                Some(Response::Err(msg)) => Err(KvsError::StringError(msg)),
                Some(_) => Err(KvsError::StringError("Invalid response".to_owned())),
                None => Err(KvsError::StringError("No response received".to_owned())),
            })
    }

    /// 从服务器中删除字符串键。
    pub fn remove(self, key: String) -> impl Future<Item = Self, Error = KvsError> {
        self.send_request(Request::Remove { key })
            .and_then(move |(resp, client)| match resp {
                Some(Response::Remove) => Ok(client),
                Some(Response::Err(msg)) => Err(KvsError::StringError(msg)),
                Some(_) => Err(KvsError::StringError("Invalid response".to_owned())),
                None => Err(KvsError::StringError("No response received".to_owned())),
            })
    }

    fn send_request(
        self,
        req: Request,
    ) -> impl Future<Item = (Option<Response>, Self), Error = KvsError> {
        let read_json = self.read_json;
        self.write_json
            .send(req)
            .and_then(move |write_json| {
                read_json
                    .into_future()
                    .map(move |(resp, read_json)| {
                        let client = KvsClient {
                            read_json,
                            write_json,
                        };
                        (resp, client)
                    })
                    .map_err(|(err, _)| err)
            })
            .map_err(|e| e.into())
    }
}