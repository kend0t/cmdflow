mod executor;
mod history;
mod storage;
mod tui;

use clap::{Parser, Subcommand};
use colored::Colorize;
use storage::{Workflow, WorkflowStore};

#[derive(Parser)]
#[command(name = "cmdflow", version, about = "Terminal workflow shortcut manager — save and run multi-command workflows")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new workflow interactively
    New,
    /// Run a workflow (interactive selection if no name given)
    Run {
        /// Name of the workflow to run
        name: Option<String>,
        /// Continue executing even if a command fails
        #[arg(long)]
        force: bool,
    },
    /// List all saved workflows
    List,
    /// Show details of a specific workflow
    Show {
        /// Workflow name
        name: String,
    },
    /// Edit an existing workflow
    Edit {
        /// Workflow name
        name: String,
    },
    /// Delete a workflow
    Delete {
        /// Workflow name (interactive if omitted)
        name: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::New) => cmd_new(),
        Some(Commands::Run { name, force }) => cmd_run(name, force),
        Some(Commands::List) => cmd_list(),
        Some(Commands::Show { name }) => cmd_show(&name),
        Some(Commands::Edit { name }) => cmd_edit(&name),
        Some(Commands::Delete { name }) => cmd_delete(name),
        None => {
            // Default: show interactive run selector
            cmd_run(None, false);
        }
    }
}

fn cmd_new() {
    let mut store = WorkflowStore::load();

    if let Some((name, desc, commands)) = tui::create_workflow_tui(None) {
        if store.get(&name).is_some() {
            println!("  {} Workflow '{}' already exists. Use 'cmdflow edit {}' instead.",
                "⚠".yellow(), name, name);
            return;
        }
        let wf = Workflow::new(name.clone(), desc, commands);
        store.add(wf);
        if let Err(e) = store.save() {
            println!("  {} Failed to save: {}", "✗".red(), e);
        } else {
            println!("\n  {} Workflow '{}' created successfully!", "✓".green().bold(), name.cyan());
        }
    } else {
        println!("  Cancelled.");
    }
}

fn cmd_run(name: Option<String>, force: bool) {
    let store = WorkflowStore::load();

    let workflow_name = match name {
        Some(n) => n,
        None => match tui::run_selector_tui(&store) {
            Some(n) => n,
            None => { println!("  Cancelled."); return; }
        },
    };

    match store.get(&workflow_name) {
        Some(wf) => {
            println!(
                "\n  {} {}\n",
                "▶ Running workflow:".bold(),
                wf.name.cyan().bold()
            );
            if !wf.description.is_empty() {
                println!("  {}\n", wf.description.dimmed());
            }
            executor::execute_workflow(&wf.commands, force);
        }
        None => {
            println!("  {} Workflow '{}' not found. Run 'cmdflow list' to see available workflows.",
                "✗".red(), workflow_name);
        }
    }
}

fn cmd_list() {
    let store = WorkflowStore::load();
    if store.workflows.is_empty() {
        println!("\n  No workflows saved. Create one with: {}\n", "cmdflow new".cyan());
        return;
    }

    println!("\n  {}\n", "Saved Workflows".bold().underline());
    for wf in &store.workflows {
        println!(
            "  {} {}  {}  ({} commands)",
            "▸".cyan(),
            wf.name.bold().white(),
            if wf.description.is_empty() { String::new() } else { format!("— {}", wf.description).dimmed().to_string() },
            wf.commands.len().to_string().yellow()
        );
    }
    println!();
}

fn cmd_show(name: &str) {
    let store = WorkflowStore::load();
    match store.get(name) {
        Some(wf) => {
            println!("\n  {} {}\n", "Workflow:".bold(), wf.name.cyan().bold());
            if !wf.description.is_empty() {
                println!("  {}: {}", "Description".dimmed(), wf.description);
            }
            println!("  {}: {}", "Created".dimmed(), wf.created_at.format("%Y-%m-%d %H:%M"));
            println!("  {}: {}\n", "Updated".dimmed(), wf.updated_at.format("%Y-%m-%d %H:%M"));
            println!("  {}", "Commands:".green().bold());
            for (i, cmd) in wf.commands.iter().enumerate() {
                println!("    {}. {}", (i + 1).to_string().yellow(), cmd);
            }
            println!();
        }
        None => {
            println!("  {} Workflow '{}' not found.", "✗".red(), name);
        }
    }
}

fn cmd_edit(name: &str) {
    let mut store = WorkflowStore::load();
    let existing = match store.get(name) {
        Some(wf) => wf.clone(),
        None => {
            println!("  {} Workflow '{}' not found.", "✗".red(), name);
            return;
        }
    };

    if let Some((new_name, desc, commands)) = tui::create_workflow_tui(Some(&existing)) {
        if let Some(wf) = store.get_mut(name) {
            wf.name = new_name.clone();
            wf.description = desc;
            wf.commands = commands;
            wf.updated_at = chrono::Local::now();
        }
        if let Err(e) = store.save() {
            println!("  {} Failed to save: {}", "✗".red(), e);
        } else {
            println!("\n  {} Workflow '{}' updated!", "✓".green().bold(), new_name.cyan());
        }
    } else {
        println!("  Cancelled.");
    }
}

fn cmd_delete(name: Option<String>) {
    let mut store = WorkflowStore::load();

    let target = match name {
        Some(n) => n,
        None => match tui::delete_confirm_tui(&store) {
            Some(n) => n,
            None => { println!("  Cancelled."); return; }
        },
    };

    if store.delete(&target) {
        if let Err(e) = store.save() {
            println!("  {} Failed to save: {}", "✗".red(), e);
        } else {
            println!("\n  {} Workflow '{}' deleted.", "✓".green().bold(), target.cyan());
        }
    } else {
        println!("  {} Workflow '{}' not found.", "✗".red(), target);
    }
}
