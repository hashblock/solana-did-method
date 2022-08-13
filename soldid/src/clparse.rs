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
                .takes_value(true)
                .default_value("~/.solwall")
                .help("Use wallet configuration [default: ~/.solwall/wallet.bor]"),
        )
        .subcommand(Command::new("did-list").about("List a wallet's keysets"))
        .subcommand(Command::new("did-create").about("Create a wallet keyset and did"))
        .subcommand(Command::new("did-rotate").about("Rotate a wallet's keyset"))
}

#[cfg(test)]
mod cli_tests {
    use std::path::PathBuf;

    use super::command_line;

    #[test]
    fn test_command_simple_did_list_pass() {
        // use super::*;
        let cmd = command_line();
        let y = cmd.get_matches_from(vec!["soldid", "did-list"]);
        let (subcmd, matches) = y.subcommand().unwrap();
        assert_eq!(subcmd, "did-list");
        assert!(!matches.args_present());
    }
    #[test]
    fn test_command_simple_did_create_pass() {
        // use super::*;
        let cmd = command_line();
        let y = cmd.get_matches_from(vec!["soldid", "did-create"]);
        let (subcmd, matches) = y.subcommand().unwrap();
        assert_eq!(subcmd, "did-create");
        assert!(!matches.args_present());
    }

    #[test]
    fn test_command_simple_did_rotate_pass() {
        // use super::*;
        let cmd = command_line();
        let y = cmd.get_matches_from(vec!["soldid", "did-rotate"]);
        let (subcmd, matches) = y.subcommand().unwrap();
        assert_eq!(subcmd, "did-rotate");
        assert!(!matches.args_present());
    }

    #[test]
    fn test_command_arg_default_wallet_pass() {
        let cmd = command_line();
        let y = cmd.get_matches_from(vec!["soldid", "did-rotate"]);
        let w: &PathBuf = y.get_one("wallet").unwrap();
        assert_eq!(PathBuf::from("~/.solwall"), *w);
    }

    #[test]
    fn test_command_arg_wallet_pass() {
        let cmd = command_line();
        let faux_dir = "~/dummy";
        let faux_path = PathBuf::from(faux_dir);
        let y = cmd.get_matches_from(vec!["soldid", "-w", faux_dir, "did-rotate"]);
        let w: &PathBuf = y.get_one("wallet").unwrap();
        assert_eq!(faux_path, *w);
    }
}
