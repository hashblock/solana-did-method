//! Command line configuration and configuration setup

use std::path::PathBuf;

use clap::{crate_description, crate_name, crate_version, value_parser, Arg, Command};

#[allow(dead_code)]
pub fn command_line() -> Command<'static> {
    Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("wallet")
                .long("wallet")
                .short('w')
                .value_parser(value_parser!(PathBuf))
                .help("Use wallet configuration [default: /.solwall/wallet.bor]"),
        )
        .subcommand(Command::new("list").about("List a wallet's keysets"))
        .subcommand(Command::new("incept").about("Create a wallet keyset and did"))
        .subcommand(Command::new("rotate").about("Rotate a wallet's keyset"))
}

#[test]
fn test_command_pass() {
    // use super::*;
    let mut cmd = command_line();

    cmd.print_help().unwrap();
}
