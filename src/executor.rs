use colored::Colorize;
use std::process::Command;

/// Result of executing a single command.
pub struct CommandResult {
    pub command: String,
    pub success: bool,
    pub exit_code: Option<i32>,
}

/// Execute a list of commands sequentially using the system shell.
/// Uses PowerShell on Windows, /bin/sh on macOS/Linux.
/// If `force` is false, stops at the first failure.
/// Returns a list of results for each executed command.
pub fn execute_workflow(commands: &[String], force: bool) -> Vec<CommandResult> {
    let total = commands.len();
    let mut results: Vec<CommandResult> = Vec::new();

    println!(
        "\n{}",
        format!("  ▶ Running {} command(s)...\n", total)
            .bold()
            .cyan()
    );

    for (i, cmd) in commands.iter().enumerate() {
        let step = format!("[{}/{}]", i + 1, total);
        println!(
            "  {} {}",
            step.bold().white(),
            cmd.dimmed()
        );

        let output = if cfg!(target_os = "windows") {
            Command::new("powershell")
                .args(["-NoProfile", "-Command", cmd])
                .output()
        } else {
            Command::new("sh")
                .args(["-c", cmd])
                .output()
        };

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);

                if !stdout.trim().is_empty() {
                    for line in stdout.lines() {
                        println!("       {}", line);
                    }
                }
                if !stderr.trim().is_empty() {
                    for line in stderr.lines() {
                        println!("       {}", line.yellow());
                    }
                }

                if out.status.success() {
                    println!("  {} {}\n", step.bold(), "✓ Done".green().bold());
                    results.push(CommandResult {
                        command: cmd.clone(),
                        success: true,
                        exit_code: out.status.code(),
                    });
                } else {
                    let code = out.status.code().unwrap_or(-1);
                    println!(
                        "  {} {}\n",
                        step.bold(),
                        format!("✗ Failed (exit code {})", code).red().bold()
                    );
                    results.push(CommandResult {
                        command: cmd.clone(),
                        success: false,
                        exit_code: out.status.code(),
                    });
                    if !force {
                        println!(
                            "  {}",
                            "⚠ Stopping due to failure. Use --force to continue regardless."
                                .yellow()
                        );
                        break;
                    }
                }
            }
            Err(e) => {
                println!(
                    "  {} {}\n",
                    step.bold(),
                    format!("✗ Error: {}", e).red().bold()
                );
                results.push(CommandResult {
                    command: cmd.clone(),
                    success: false,
                    exit_code: None,
                });
                if !force {
                    println!(
                        "  {}",
                        "⚠ Stopping due to error. Use --force to continue regardless."
                            .yellow()
                    );
                    break;
                }
            }
        }
    }

    // Summary
    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = results.iter().filter(|r| !r.success).count();

    if failed == 0 {
        println!(
            "\n  {}",
            format!("✅ All {} command(s) completed successfully!", succeeded)
                .green()
                .bold()
        );
    } else {
        println!(
            "\n  {}",
            format!(
                "⚠ {} succeeded, {} failed out of {} command(s)",
                succeeded,
                failed,
                results.len()
            )
            .yellow()
            .bold()
        );
    }

    results
}
