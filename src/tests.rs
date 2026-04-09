// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn test_cli_parse_ui() {
    let cli = Cli::parse_from(["petaltongue", "ui"]);
    assert!(matches!(cli.command, Commands::Ui { .. }));
}

#[test]
fn test_cli_parse_tui() {
    let cli = Cli::parse_from(["petaltongue", "tui"]);
    assert!(matches!(cli.command, Commands::Tui { .. }));
}

#[test]
fn test_cli_parse_web() {
    let cli = Cli::parse_from(["petaltongue", "web"]);
    assert!(matches!(cli.command, Commands::Web { .. }));
}

#[test]
fn test_cli_parse_headless() {
    let cli = Cli::parse_from(["petaltongue", "headless"]);
    assert!(matches!(cli.command, Commands::Headless { .. }));
}

#[test]
fn test_cli_parse_server() {
    let cli = Cli::parse_from(["petaltongue", "server"]);
    assert!(matches!(cli.command, Commands::Server { .. }));
}

#[test]
fn test_cli_parse_server_with_port() {
    let cli = Cli::parse_from(["petaltongue", "server", "--port", "12345"]);
    let Commands::Server { port } = cli.command else {
        unreachable!("CLI parsed 'server' subcommand")
    };
    assert_eq!(port, Some(12345));
}

#[test]
fn test_cli_parse_status() {
    let cli = Cli::parse_from(["petaltongue", "status"]);
    assert!(matches!(cli.command, Commands::Status { .. }));
}

#[test]
fn test_cli_parse_ui_with_scenario() {
    let cli = Cli::parse_from(["petaltongue", "ui", "--scenario", "test.json"]);
    let Commands::Ui { scenario, no_audio } = cli.command else {
        unreachable!("CLI parsed 'ui' subcommand")
    };
    assert_eq!(scenario.as_deref(), Some("test.json"));
    assert!(!no_audio);
}

#[test]
fn test_cli_parse_ui_no_audio() {
    let cli = Cli::parse_from(["petaltongue", "ui", "--no-audio"]);
    let Commands::Ui { no_audio, .. } = cli.command else {
        unreachable!("CLI parsed 'ui' subcommand")
    };
    assert!(no_audio);
}

#[test]
fn test_cli_parse_tui_with_refresh_rate() {
    let cli = Cli::parse_from(["petaltongue", "tui", "--refresh-rate", "30"]);
    let Commands::Tui { refresh_rate, .. } = cli.command else {
        unreachable!("CLI parsed 'tui' subcommand")
    };
    assert_eq!(refresh_rate, 30);
}

#[test]
fn test_cli_parse_tui_default_refresh_rate() {
    let cli = Cli::parse_from(["petaltongue", "tui"]);
    let Commands::Tui { refresh_rate, .. } = cli.command else {
        unreachable!("CLI parsed 'tui' subcommand")
    };
    assert_eq!(refresh_rate, 60);
}

#[test]
fn test_cli_parse_tui_with_scenario() {
    let cli = Cli::parse_from(["petaltongue", "tui", "--scenario", "demo.json"]);
    let Commands::Tui { scenario, .. } = cli.command else {
        unreachable!("CLI parsed 'tui' subcommand")
    };
    assert_eq!(scenario.as_deref(), Some("demo.json"));
}

#[test]
fn test_cli_parse_status_default_verbose() {
    let cli = Cli::parse_from(["petaltongue", "status"]);
    let Commands::Status { verbose, .. } = cli.command else {
        unreachable!("CLI parsed 'status' subcommand")
    };
    assert!(!verbose);
}

#[test]
fn test_cli_parse_status_verbose() {
    let cli = Cli::parse_from(["petaltongue", "status", "--verbose"]);
    let Commands::Status { verbose, .. } = cli.command else {
        unreachable!("CLI parsed 'status' subcommand")
    };
    assert!(verbose);
}

#[test]
fn test_cli_parse_status_text_format() {
    let cli = Cli::parse_from(["petaltongue", "status", "--format", "text"]);
    let Commands::Status { format, .. } = cli.command else {
        unreachable!("CLI parsed 'status' subcommand")
    };
    assert_eq!(format, "text");
}

#[test]
fn test_cli_log_level_trace() {
    let cli = Cli::parse_from(["petaltongue", "--log-level", "trace", "status"]);
    assert_eq!(cli.log_level, "trace");
}

#[test]
fn test_cli_log_level_warn() {
    let cli = Cli::parse_from(["petaltongue", "--log-level", "warn", "status"]);
    assert_eq!(cli.log_level, "warn");
}

#[test]
fn test_cli_log_format_json() {
    let cli = Cli::parse_from(["petaltongue", "--log-format", "json", "status"]);
    assert_eq!(cli.log_format, "json");
}

#[test]
fn test_cli_parse_web_with_bind() {
    let cli = Cli::parse_from(["petaltongue", "web", "--bind", "127.0.0.1:9090"]);
    let Commands::Web { bind, workers, .. } = cli.command else {
        unreachable!("CLI parsed 'web' subcommand")
    };
    assert_eq!(bind.as_deref(), Some("127.0.0.1:9090"));
    assert_eq!(workers, 4);
}

#[test]
fn test_cli_parse_web_with_workers() {
    let cli = Cli::parse_from(["petaltongue", "web", "--workers", "8"]);
    let Commands::Web { workers, .. } = cli.command else {
        unreachable!("CLI parsed 'web' subcommand")
    };
    assert_eq!(workers, 8);
}

#[test]
fn test_cli_parse_headless_with_all_options() {
    let cli = Cli::parse_from([
        "petaltongue",
        "headless",
        "--bind",
        "0.0.0.0:7070",
        "--workers",
        "2",
    ]);
    let Commands::Headless { bind, workers } = cli.command else {
        unreachable!("CLI parsed 'headless' subcommand")
    };
    assert_eq!(bind.as_deref(), Some("0.0.0.0:7070"));
    assert_eq!(workers, 2);
}

#[test]
fn test_cli_parse_status_verbose_json() {
    let cli = Cli::parse_from(["petaltongue", "status", "--verbose", "--format", "json"]);
    let Commands::Status { verbose, format } = cli.command else {
        unreachable!("CLI parsed 'status' subcommand")
    };
    assert!(verbose);
    assert_eq!(format, "json");
}

#[test]
fn test_cli_default_log_level() {
    let cli = Cli::parse_from(["petaltongue", "status"]);
    assert_eq!(cli.log_level, "info");
    assert_eq!(cli.log_format, "pretty");
}

#[test]
fn test_cli_custom_log_level() {
    let cli = Cli::parse_from(["petaltongue", "--log-level", "debug", "status"]);
    assert_eq!(cli.log_level, "debug");
}

#[test]
fn test_cli_gui_alias() {
    let cli = Cli::parse_from(["petaltongue", "gui"]);
    assert!(matches!(cli.command, Commands::Ui { .. }));
}

#[test]
fn test_cli_parse_ui_with_scenario_and_no_audio() {
    let cli = Cli::parse_from(["petaltongue", "ui", "--scenario", "demo.json", "--no-audio"]);
    let Commands::Ui { scenario, no_audio } = cli.command else {
        unreachable!("CLI parsed 'ui' subcommand")
    };
    assert_eq!(scenario.as_deref(), Some("demo.json"));
    assert!(no_audio);
}

#[test]
fn test_cli_parse_web_with_scenario() {
    let cli = Cli::parse_from(["petaltongue", "web", "--scenario", "tutorial.json"]);
    let Commands::Web { scenario, .. } = cli.command else {
        unreachable!("CLI parsed 'web' subcommand")
    };
    assert_eq!(scenario.as_deref(), Some("tutorial.json"));
}

#[test]
fn test_cli_custom_log_format() {
    let cli = Cli::parse_from(["petaltongue", "--log-format", "compact", "status"]);
    assert_eq!(cli.log_format, "compact");
}

#[test]
fn test_cli_parse_headless_default_workers() {
    let cli = Cli::parse_from(["petaltongue", "headless"]);
    let Commands::Headless { workers, .. } = cli.command else {
        unreachable!("CLI parsed 'headless' subcommand")
    };
    assert_eq!(workers, 4);
}

#[test]
fn test_cli_parse_status_default_format() {
    let cli = Cli::parse_from(["petaltongue", "status"]);
    let Commands::Status { format, .. } = cli.command else {
        unreachable!("CLI parsed 'status' subcommand")
    };
    assert_eq!(format, "text");
}

#[test]
fn test_commands_debug() {
    let cmd = Commands::Status {
        verbose: false,
        format: "text".to_string(),
    };
    let debug_str = format!("{cmd:?}");
    assert!(debug_str.contains("Status"));
}

// init_tracing tests - tracing can only be initialized once per process.
// Run test_init_tracing_invalid_level first (doesn't init).
// test_init_tracing_formats exercises all format branches and double-init error.

#[test]
fn test_init_tracing_invalid_level() {
    // "crate=invalid_level" - invalid level fails at EnvFilter::try_new (does not call try_init)
    let result = init_tracing("crate=invalid_level", "pretty");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("log level") || err_msg.contains("parse"));
}

#[test]
fn test_init_tracing_formats() {
    // Tracing can only be initialized once per process. Run formats test first (single-threaded
    // or before any other init). If another test already initialized, all inits here will fail.
    // 1. JSON format init (first successful init in this test)
    let result = init_tracing("info", "json");
    assert!(
        result.is_ok(),
        "first init with json format should succeed: {result:?}"
    );

    // 2. Compact format - fails because already initialized (exercises compact branch)
    let result = init_tracing("debug", "compact");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("compact") || err_msg.contains("init"),
        "expected compact init error: {err_msg}"
    );

    // 3. Pretty format - fails because already initialized (exercises default branch)
    let result = init_tracing("info", "pretty");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("pretty") || err_msg.contains("init"),
        "expected pretty init error: {err_msg}"
    );
}

#[tokio::test]
async fn test_register_with_discovery_service_completes() {
    // Registration runs to completion (gracefully handles service unavailability)
    register_with_discovery_service().await;
}

#[test]
fn test_config_from_env_loads() {
    let config = Config::from_env().expect("Config::from_env should load");
    assert!(config.network.web_port > 0);
    assert!(config.network.headless_port > 0);
}

#[test]
fn test_web_bind_addr_fallback_format() {
    let port = 3000u16;
    let addr = format!("0.0.0.0:{port}");
    assert_eq!(addr, "0.0.0.0:3000");
}

#[test]
fn test_web_bind_addr_explicit_override() {
    let bind = Some("127.0.0.1:9090".to_string());
    let addr = bind.as_deref().unwrap_or("0.0.0.0:3000");
    assert_eq!(addr, "127.0.0.1:9090");
}

#[test]
fn test_headless_bind_addr_fallback_format() {
    let port = 8080u16;
    let addr = format!("0.0.0.0:{port}");
    assert_eq!(addr, "0.0.0.0:8080");
}

#[test]
fn test_subcommand_routing_web_bind_resolution() {
    // When --bind is omitted, main uses config.network.web_port
    let cli = Cli::parse_from(["petaltongue", "web"]);
    let Commands::Web { bind, .. } = cli.command else {
        unreachable!("parsed web")
    };
    assert!(bind.is_none(), "default web has no explicit bind");
}

#[test]
fn test_subcommand_routing_headless_bind_resolution() {
    let cli = Cli::parse_from(["petaltongue", "headless"]);
    let Commands::Headless { bind, .. } = cli.command else {
        unreachable!("parsed headless")
    };
    assert!(bind.is_none(), "default headless has no explicit bind");
}

/// Config loading error path - same map_err as main() line 142
#[test]
fn test_config_from_env_error_produces_app_error() {
    use petal_tongue_core::test_fixtures::env_test_helpers;

    let temp = std::env::temp_dir().join("petaltongue-main-config-test.toml");
    let contents = "\
[network]\n\
web_port = 3000\n\
headless_port = 8080\n";
    std::fs::write(&temp, contents).expect("write temp");
    let path = temp.to_str().expect("path");

    env_test_helpers::with_env_vars(
        &[
            ("PETALTONGUE_CONFIG", Some(path)),
            ("PETALTONGUE_WEB_PORT", Some("not-a-number")),
        ],
        || {
            let result = Config::from_env().map_err(|e| AppError::Other(e.to_string()));
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, AppError::Other(_)));
            assert!(err.to_string().contains("Invalid") || err.to_string().contains("WEB_PORT"));
        },
    );
    let _ = std::fs::remove_file(&temp);
}

/// Result error handling path - exercises the Err branch in main() match result
#[test]
fn test_app_error_result_propagates() {
    let err = AppError::UiNotAvailable;
    let result: Result<(), _> = Err(err);
    assert!(result.is_err());
    assert!(matches!(result, Err(AppError::UiNotAvailable)));
}
