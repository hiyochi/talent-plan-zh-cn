use clap::AppSettings;
use kvs::{KvsClient, Result};
use std::net::SocketAddr;
use std::process::exit;
use structopt::StructOpt;

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:4000";
const ADDRESS_FORMAT: &str = "IP:PORT";

#[derive(StructOpt, Debug)]
#[structopt(
    name = "kvs-client",
    raw(global_settings = "&[\
                           AppSettings::DisableHelpSubcommand,\
                           AppSettings::VersionlessSubcommands]")
)]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "get", about = "获取指定字符串键的字符串值")]
    Get {
        #[structopt(name = "KEY", help = "一个字符串键")]
        key: String,
        #[structopt(
            long,
            help = "设置服务器地址",
            raw(value_name = "ADDRESS_FORMAT"),
            raw(default_value = "DEFAULT_LISTENING_ADDRESS"),
            parse(try_from_str)
        )]
        addr: SocketAddr,
    },
    #[structopt(name = "set", about = "将字符串键的值设置为字符串")]
    Set {
        #[structopt(name = "KEY", help = "一个字符串键")]
        key: String,
        #[structopt(name = "VALUE", help = "键的字符串值")]
        value: String,
        #[structopt(
            long,
            help = "设置服务器地址",
            raw(value_name = "ADDRESS_FORMAT"),
            raw(default_value = "DEFAULT_LISTENING_ADDRESS"),
            parse(try_from_str)
        )]
        addr: SocketAddr,
    },
    #[structopt(name = "rm", about = "删除指定的字符串键")]
    Remove {
        #[structopt(name = "KEY", help = "一个字符串键")]
        key: String,
        #[structopt(
            long,
            help = "设置服务器地址",
            raw(value_name = "ADDRESS_FORMAT"),
            raw(default_value = "DEFAULT_LISTENING_ADDRESS"),
            parse(try_from_str)
        )]
        addr: SocketAddr,
    },
}

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = run(opt) {
        eprintln!("{}", e);
        exit(1);
    }
}

fn run(opt: Opt) -> Result<()> {
    match opt.command {
        Command::Get { key, addr } => {
            let mut client = KvsClient::connect(addr)?;
            if let Some(value) = client.get(key)? {
                println!("{}", value);
            } else {
                println!("键未找到");
            }
        }
        Command::Set { key, value, addr } => {
            let mut client = KvsClient::connect(addr)?;
            client.set(key, value)?;
        }
        Command::Remove { key, addr } => {
            let mut client = KvsClient::connect(addr)?;
            client.remove(key)?;
        }
    }
    Ok(())
}