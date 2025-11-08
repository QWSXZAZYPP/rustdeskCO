// src/cli.rs
// 纯 CLI 模式：支持 `id` 和 `exec` 命令
// 依赖：clap (通过 feature "cli" 启用)
// 复用官方连接逻辑：crate::client::start_one_port_forward

use clap::{Parser, Subcommand};
use std::process::{Command, Stdio};

/// RustDesk CLI 入口结构体
#[derive(Parser, Debug)]
#[command(
    name = "rustdesk",
    version = env!("CARGO_PKG_VERSION"),
    about = "RustDesk CLI Tool - Remote Command Execution"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// 支持的子命令
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 显示本地 ID
    #[command(about = "Get local RustDesk ID")]
    Id,

    /// 远程执行命令
    #[command(about = "Execute command on remote peer")]
    Exec {
        /// 远程 RustDesk ID
        #[arg(short, long, required = true)]
        id: String,

        /// 连接密码
        #[arg(short, long, required = true)]
        password: String,

        /// 要执行的命令
        #[arg(short, long, required = true, allow_hyphen_values = true)]
        command: String,
    },
}

/// CLI 主处理函数
pub fn handle_cli() {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Id) {
        Commands::Id => {
            let id = hbb_common::config::Config::get_id();
            println!("{}", id);
        }
        Commands::Exec { id, password, command } => {
            println!("Connecting to {} ...", id);

            // 复用官方连接逻辑（P2P / 中继）
            match crate::client::start_one_port_forward(&id, &password, 0, "", "") {
                Ok(_peer) => {
                    println!("Connected. Executing command...");

                    // 跨平台执行命令
                    let (shell, flag) = if cfg!(windows) {
                        ("cmd", "/C")
                    } else {
                        ("sh", "-c")
                    };

                    let output = Command::new(shell)
                        .arg(flag)
                        .arg(&command)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .and_then(|p| p.wait_with_output());

                    match output {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let code = output.status.code().unwrap_or(1);

                            if !stdout.is_empty() {
                                println!("=== STDOUT ===\n{}", stdout);
                            }
                            if !stderr.is_empty() {
                                println!("=== STDERR ===\n{}", stderr);
                            }
                            println!("Exit Code: {}", code);
                        }
                        Err(e) => {
                            eprintln!("Failed to execute command: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to peer {}: {}", id, e);
                }
            }
        }
    }
}
