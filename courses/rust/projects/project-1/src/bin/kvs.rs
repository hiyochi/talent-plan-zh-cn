use clap::{App, AppSettings, Arg, SubCommand};
use std::process::exit;

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(
            SubCommand::with_name("set")
                .about("设置字符串键的值为字符串")
                .arg(Arg::with_name("KEY").help("一个字符串键").required(true))
                .arg(
                    Arg::with_name("VALUE")
                        .help("键的字符串值")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("获取指定字符串键的字符串值")
                .arg(Arg::with_name("KEY").help("一个字符串键").required(true)),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("删除指定的键")
                .arg(Arg::with_name("KEY").help("一个字符串键").required(true)),
        )
        .get_matches();

    match matches.subcommand() {
        ("set", Some(_matches)) => {
            eprintln!("未实现");
            exit(1);
        }
        ("get", Some(_matches)) => {
            eprintln!("未实现");
            exit(1);
        }
        ("rm", Some(_matches)) => {
            eprintln!("未实现");
            exit(1);
        }
        _ => unreachable!(),
    }
}