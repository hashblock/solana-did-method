//! Command line configuration and configuration setup

use std::path::PathBuf;

use clap::{crate_description, crate_name, crate_version, value_parser, Arg, Command};

pub const DID_LIST: &str = "did-list";
pub const DID_CREATE: &str = "did-create";
pub const DID_ROTATE: &str = "did-rotate";
pub const DID_DECOMMISION: &str = "did-decommision";
pub const DID_CLOSE: &str = "did-close";

#[allow(dead_code)]
pub fn command_line() -> Command<'static> {
    Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("wallet")
                .long("did-wallet")
                .short('w')
                .global(true)
                .value_parser(value_parser!(PathBuf))
                .takes_value(true)
                .default_value("~/.solwall")
                .help("Use wallet configuration in path [default: ~/.solwall]"),
        )
        .subcommand(Command::new(DID_LIST).about("List a wallet's keysets"))
        .subcommand(
            Command::new(DID_CREATE)
                .about("Create a wallet keyset and did")
                .arg(
                    Arg::new("name")
                        .short('n')
                        .takes_value(true)
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("Set the new managed keys of the DID to a familiar name"),
                )
                .arg(
                    Arg::new("keys")
                        .short('k')
                        .takes_value(true)
                        .default_value("2")
                        .value_parser(value_parser!(i8))
                        .help("Set the number of keypairs to generate for the DID"),
                )
                .arg(
                    Arg::new("threshold")
                        .short('t')
                        .takes_value(true)
                        .default_value("1")
                        .value_parser(value_parser!(i8))
                        .help("Set the signing threshold to modify the DID document"),
                ),
        )
        .subcommand(Command::new(DID_ROTATE).about("Rotate a wallet's keyset"))
        .subcommand(Command::new(DID_DECOMMISION).about("Decommision a wallet's keyset"))
        .subcommand(
            Command::new(DID_CLOSE)
                .about("Close a DID account without removing keyset")
                .arg(
                    Arg::new("pda")
                        .short('p')
                        .takes_value(true)
                        .help("PDA pubkey string"),
                ),
        )
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
        assert!(matches.args_present());
    }
    #[test]
    fn test_command_simple_did_create_pass() {
        // use super::*;
        let cmd = command_line();
        let y = cmd.get_matches_from(vec!["soldid", "did-create", "-n", "Alice"]);
        let (subcmd, matches) = y.subcommand().unwrap();
        assert_eq!(subcmd, "did-create");
        assert_eq!(*matches.get_one::<i8>("keys").unwrap(), 2);
        assert_eq!(*matches.get_one::<i8>("threshold").unwrap(), 1);
    }

    #[test]
    fn test_command_simple_did_rotate_pass() {
        // use super::*;
        let cmd = command_line();
        let y = cmd.get_matches_from(vec!["soldid", "did-rotate"]);
        let (subcmd, _matches) = y.subcommand().unwrap();
        assert_eq!(subcmd, "did-rotate");
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
        // println!("{:?}", y.value_source("wallet").unwrap());
        // assert_eq!(y.occurrences_of("wallet"), 0);

        let w: &PathBuf = y.get_one("wallet").unwrap();
        assert_eq!(faux_path, *w);
    }
}
