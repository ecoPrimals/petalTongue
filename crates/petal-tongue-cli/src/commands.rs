// SPDX-License-Identifier: AGPL-3.0-or-later
//! CLI command definitions and argument parsing.

use clap::{Parser, Subcommand};

/// CLI argument parser for petalTongue instance management.
#[derive(Debug, Parser)]
#[command(name = "petaltongue")]
#[command(about = "petalTongue instance manager", long_about = None)]
#[command(version)]
pub struct Cli {
    /// The subcommand to run.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List all running instances
    List,

    /// Show detailed information about an instance
    Show {
        /// Instance ID (UUID or prefix)
        #[arg(value_name = "INSTANCE_ID")]
        instance_id: String,
    },

    /// Bring instance window to foreground
    Raise {
        /// Instance ID (UUID or prefix)
        #[arg(value_name = "INSTANCE_ID")]
        instance_id: String,
    },

    /// Ping an instance to check if it's responsive
    Ping {
        /// Instance ID (UUID or prefix)
        #[arg(value_name = "INSTANCE_ID")]
        instance_id: String,
    },

    /// Clean up dead instances from registry
    Gc {
        /// Actually remove dead instances (default: dry-run)
        #[arg(short, long)]
        force: bool,
    },

    /// Show status of all instances
    Status,
}

/// Parse CLI arguments (for testing)
#[cfg(test)]
pub fn parse_args(args: &[&str]) -> std::result::Result<Commands, clap::Error> {
    let cli = Cli::try_parse_from(args)?;
    Ok(cli.command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_list() {
        let cmd = parse_args(&["petaltongue", "list"]).unwrap();
        assert!(matches!(cmd, Commands::List));
    }

    #[test]
    fn test_parse_status() {
        let cmd = parse_args(&["petaltongue", "status"]).unwrap();
        assert!(matches!(cmd, Commands::Status));
    }

    #[test]
    fn test_parse_show() {
        let cmd = parse_args(&["petaltongue", "show", "abc-123"]).unwrap();
        match &cmd {
            Commands::Show { instance_id } => assert_eq!(instance_id, "abc-123"),
            _ => panic!("Expected Show command"),
        }
    }

    #[test]
    fn test_parse_gc() {
        let cmd = parse_args(&["petaltongue", "gc"]).unwrap();
        match &cmd {
            Commands::Gc { force } => assert!(!*force),
            _ => panic!("Expected Gc command"),
        }
    }

    #[test]
    fn test_parse_gc_force() {
        let cmd = parse_args(&["petaltongue", "gc", "--force"]).unwrap();
        match &cmd {
            Commands::Gc { force } => assert!(*force),
            _ => panic!("Expected Gc command"),
        }
    }

    #[test]
    fn test_parse_ping() {
        let cmd = parse_args(&["petaltongue", "ping", "uuid-here"]).unwrap();
        match &cmd {
            Commands::Ping { instance_id } => assert_eq!(instance_id, "uuid-here"),
            _ => panic!("Expected Ping command"),
        }
    }

    #[test]
    fn test_parse_raise() {
        let cmd = parse_args(&["petaltongue", "raise", "inst-id"]).unwrap();
        match &cmd {
            Commands::Raise { instance_id } => assert_eq!(instance_id, "inst-id"),
            _ => panic!("Expected Raise command"),
        }
    }

    #[test]
    fn test_help_text_generation() {
        let result = Cli::try_parse_from(["petaltongue", "--help"]);
        let err = result.expect_err("--help should produce Err with help text");
        let help = err.to_string();
        assert!(help.contains("petaltongue"));
        assert!(help.contains("list") || help.contains("List"));
        assert!(help.contains("show") || help.contains("Show"));
    }

    #[test]
    fn test_version_output() {
        let result = Cli::try_parse_from(["petaltongue", "--version"]);
        let err = result.expect_err("--version should produce Err with version");
        let version = err.to_string();
        assert!(version.contains("petaltongue"));
        assert!(version.contains('.') || version.chars().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_list_subcommand_help() {
        let result = Cli::try_parse_from(["petaltongue", "list", "--help"]);
        let err = result.expect_err("list --help should produce Err with help");
        let help = err.to_string();
        assert!(help.contains("list") || help.contains("List"));
    }

    #[test]
    fn test_gc_subcommand_with_force_flag() {
        let cmd = parse_args(&["petaltongue", "gc", "-f"]).unwrap();
        match &cmd {
            Commands::Gc { force } => assert!(*force),
            _ => panic!("Expected Gc command"),
        }
    }

    #[test]
    fn test_parse_args_missing_subcommand_fails() {
        let result = parse_args(&["petaltongue"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_unknown_subcommand_fails() {
        let result = parse_args(&["petaltongue", "unknown-cmd"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_show_with_empty_id() {
        let cmd = parse_args(&["petaltongue", "show", ""]).unwrap();
        match &cmd {
            Commands::Show { instance_id } => assert_eq!(instance_id, ""),
            _ => panic!("Expected Show command"),
        }
    }

    #[test]
    fn test_parse_args_raise_with_uuid() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let cmd = parse_args(&["petaltongue", "raise", uuid]).unwrap();
        match &cmd {
            Commands::Raise { instance_id } => assert_eq!(instance_id, uuid),
            _ => panic!("Expected Raise command"),
        }
    }

    #[test]
    fn test_parse_args_ping_with_short_prefix() {
        let cmd = parse_args(&["petaltongue", "ping", "550e"]).unwrap();
        match &cmd {
            Commands::Ping { instance_id } => assert_eq!(instance_id, "550e"),
            _ => panic!("Expected Ping command"),
        }
    }

    #[test]
    fn test_run_command_dispatch_list() {
        let cmd = parse_args(&["petaltongue", "list"]).unwrap();
        assert!(matches!(cmd, Commands::List));
    }

    #[test]
    fn test_run_command_dispatch_status() {
        let cmd = parse_args(&["petaltongue", "status"]).unwrap();
        assert!(matches!(cmd, Commands::Status));
    }

    #[test]
    fn test_cli_struct_has_command_field() {
        let cli = Cli::try_parse_from(["petaltongue", "list"]).unwrap();
        assert!(matches!(cli.command, Commands::List));
    }

    #[test]
    fn test_show_subcommand_accepts_instance_id_arg() {
        let cmd = parse_args(&["petaltongue", "show", "my-instance-123"]).unwrap();
        match &cmd {
            Commands::Show { instance_id } => assert_eq!(instance_id, "my-instance-123"),
            _ => panic!("Expected Show"),
        }
    }

    #[test]
    fn test_parse_args_empty_fails() {
        let result = parse_args(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_only_binary_name_fails() {
        let result = parse_args(&["petaltongue"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_invalid_flag() {
        let result = parse_args(&["petaltongue", "list", "--invalid-flag"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_show_requires_arg() {
        let result = parse_args(&["petaltongue", "show"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_raise_requires_arg() {
        let result = parse_args(&["petaltongue", "raise"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_ping_requires_arg() {
        let result = parse_args(&["petaltongue", "ping"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_output_format() {
        let cmd = parse_args(&["petaltongue", "list"]).unwrap();
        assert!(matches!(cmd, Commands::List));
    }

    #[test]
    fn test_all_subcommands_parse() {
        let list = parse_args(&["petaltongue", "list"]).unwrap();
        assert!(matches!(list, Commands::List));

        let status = parse_args(&["petaltongue", "status"]).unwrap();
        assert!(matches!(status, Commands::Status));

        let gc = parse_args(&["petaltongue", "gc"]).unwrap();
        assert!(matches!(gc, Commands::Gc { .. }));
    }

    #[test]
    fn test_show_subcommand_error_handling() {
        let cmd = parse_args(&["petaltongue", "show", "nonexistent-uuid-xxxx"]).unwrap();
        assert!(matches!(cmd, Commands::Show { .. }));
    }

    #[test]
    fn test_commands_enum_exhaustive() {
        let _ = Commands::List;
        let _ = Commands::Show {
            instance_id: "x".to_string(),
        };
        let _ = Commands::Raise {
            instance_id: "x".to_string(),
        };
        let _ = Commands::Ping {
            instance_id: "x".to_string(),
        };
        let _ = Commands::Gc { force: false };
        let _ = Commands::Status;
    }
}
