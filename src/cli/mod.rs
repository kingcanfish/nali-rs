//! CLI module for nali-rs
//!
//! This module handles command line argument parsing and query logic.

use crate::config::AppConfig;
use crate::database::DatabaseManager;
use crate::download::Downloader;
use crate::entity::{parser, formatter, EntityType};
use crate::error::Result;
use clap::Parser;
use std::io::{self, BufRead, Write};
use std::net::IpAddr;

#[derive(Parser, Debug)]
#[command(name = "nali-rs")]
#[command(version, about = "一个查询IP地理信息和CDN提供商的离线终端工具 - Rust实现")]
#[command(long_about = "nali-rs 是 nali 的 Rust 实现版本\n\n\
    支持从命令行参数、管道或交互模式查询 IP 地理位置和 CDN 提供商信息。\n\n\
    示例:\n  \
    $ nali-rs 1.2.3.4\n  \
    $ echo \"Server IP: 8.8.8.8\" | nali-rs\n  \
    $ dig google.com | nali-rs\n  \
    $ nali-rs --json 1.2.3.4\n  \
    $ nali-rs update\n  \
    $ nali-rs update qqwry")]
pub struct Cli {
    /// IP地址或域名列表（如果没有提供，则从标准输入读取）
    #[arg(value_name = "QUERY")]
    pub queries: Vec<String>,

    /// 输出JSON格式
    #[arg(short, long)]
    pub json: bool,

    /// 使用GBK解码器（用于中文数据库）
    #[arg(short, long)]
    pub gbk: bool,

    /// 显示详细信息
    #[arg(short, long)]
    pub verbose: bool,

    /// 更新数据库 (update [database_name])
    #[arg(long)]
    pub update: bool,
}

impl Cli {
    pub async fn run(&self, mut config: AppConfig) -> Result<()> {
        // Handle update command first
        if self.update {
            return self.handle_update(&config).await;
        }

        // Apply CLI options to config
        if self.json {
            config.output.json = true;
        }
        if self.gbk {
            config.output.use_gbk = true;
        }
        if self.verbose {
            config.global.verbose = true;
        }

        // Create database manager
        let db_manager = DatabaseManager::new(config.clone());

        if !self.queries.is_empty() {
            // Query from command line arguments
            self.process_queries_from_args(&db_manager, &config).await?;
        } else {
            // Query from stdin (pipe mode or interactive mode)
            self.process_queries_from_stdin(&db_manager, &config).await?;
        }

        Ok(())
    }

    /// Process queries from command line arguments
    async fn process_queries_from_args(&self, db_manager: &DatabaseManager, config: &AppConfig) -> Result<()> {
        for query in &self.queries {
            // Try to parse as IP address
            if let Ok(ip) = query.parse::<IpAddr>() {
                self.query_and_print_ip(ip, db_manager, config).await?;
            } else {
                // Treat as domain or text
                self.query_and_print_text(query, db_manager, config).await?;
            }
        }
        Ok(())
    }

    /// Process queries from stdin (pipe or interactive mode)
    async fn process_queries_from_stdin(&self, db_manager: &DatabaseManager, config: &AppConfig) -> Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        // Check if stdin is a TTY (interactive mode)
        if atty::is(atty::Stream::Stdin) {
            // Interactive mode
            println!("nali-rs interactive mode (输入 quit 或 Ctrl+D 退出)");

            for line in stdin.lock().lines() {
                let line = line?;
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    continue;
                }

                if trimmed == "quit" || trimmed == "exit" {
                    break;
                }

                // Process the line
                let result = self.process_line(trimmed, db_manager, config).await?;
                println!("{}", result);
                stdout.flush()?;
            }
        } else {
            // Pipe mode - read from stdin and enrich with geolocation info
            // Note: We preserve line endings to match the original text format
            // The lines() iterator strips \n, but we need to add them back
            use std::io::Read;
            let mut buffer = String::new();
            stdin.lock().read_to_string(&mut buffer)?;

            for line in buffer.lines() {
                // Re-add the newline that lines() strips
                let line_with_newline = format!("{}\n", line);
                let result = self.process_line(&line_with_newline, db_manager, config).await?;
                print!("{}", result);  // Use print! not println! since line already has \n
            }
        }

        Ok(())
    }

    /// Process a single line of text
    async fn process_line(&self, line: &str, db_manager: &DatabaseManager, config: &AppConfig) -> Result<String> {
        // Parse entities from the line
        let mut entities = parser::parse_line(line);

        // Enrich entities with geolocation/CDN information
        for entity in &mut entities.entities {
            match entity.entity_type {
                EntityType::IPv4 | EntityType::IPv6 => {
                    if let Some(ip) = entity.as_ip() {
                        if let Ok(Some(geo)) = db_manager.query_ip(ip).await {
                            entity.geo_info = Some(geo);
                            entity.source = Some(config.database.ipv4_database.clone());
                        }
                    }
                }
                EntityType::Domain => {
                    if let Ok(Some(cdn)) = db_manager.query_cdn(&entity.text).await {
                        entity.cdn_info = Some(cdn);
                        entity.source = Some(config.database.cdn_database.clone());
                    }
                }
                EntityType::Plain => {}
            }
        }

        // Build complete entities with plain text segments
        let complete = parser::build_complete_entities(line, entities);

        // Format output
        if config.output.json {
            formatter::format_json(&complete)
                .map_err(|e| crate::error::NaliError::JsonError(e))
        } else {
            Ok(formatter::format_text(&complete, config.output.enable_colors))
        }
    }

    /// Query and print a single IP
    async fn query_and_print_ip(&self, ip: IpAddr, db_manager: &DatabaseManager, config: &AppConfig) -> Result<()> {
        match db_manager.query_ip(ip).await {
            Ok(Some(geo)) => {
                if config.output.json {
                    let json = serde_json::to_string_pretty(&geo)?;
                    println!("{}", json);
                } else {
                    let info = formatter::format_geo_info_compact(&geo);
                    println!("{} -> {}", ip, info);
                }
            }
            Ok(None) => {
                println!("{} -> [Not found]", ip);
            }
            Err(e) => {
                eprintln!("Query failed: {}", e);
            }
        }
        Ok(())
    }

    /// Query and print text (may contain IPs and domains)
    async fn query_and_print_text(&self, text: &str, db_manager: &DatabaseManager, config: &AppConfig) -> Result<()> {
        let result = self.process_line(text, db_manager, config).await?;
        println!("{}", result);
        Ok(())
    }

    /// Handle database update command
    async fn handle_update(&self, config: &AppConfig) -> Result<()> {
        let downloader = Downloader::new()?;

        if self.queries.is_empty() {
            // No specific database specified, update all
            println!("Updating all databases...\n");
            downloader.download_all(config).await?;
        } else {
            // Update specific databases
            for db_name in &self.queries {
                match downloader.update_database(config, db_name).await {
                    Ok(_) => {},
                    Err(e) => {
                        eprintln!("✗ Failed to update {}: {}", db_name, e);
                    }
                }
                println!();
            }
        }

        Ok(())
    }
}
