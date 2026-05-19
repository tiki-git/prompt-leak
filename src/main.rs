// main.rs — point d'entrée de prompt-leak

mod attack;
mod config;
mod report;

use clap::Parser;
use colored::*;

use attack::session;
use config::Config;

#[derive(Parser)]
#[command(
    name    = "prompt-leak",
    about   = "Agent d'exfiltration de system prompt — Kali Linux LLM red team",
    version = "0.1.0",
)]
pub struct Cli {
    /// URL de base de l'API LLM (ex: http://localhost:11434)
    #[arg(short, long)]
    pub target: String,

    /// Clé API si requise (OpenAI-compatible)
    #[arg(short, long)]
    pub key: Option<String>,

    /// Nom du modèle à attaquer
    #[arg(short, long, default_value = "llama3")]
    pub model: String,

    /// Chemin du rapport de sortie
    #[arg(short, long, default_value = "report.md")]
    pub output: String,

    /// Passe toutes les confirmations automatiquement
    #[arg(long)]
    pub auto: bool,

    /// Affiche les réponses complètes du LLM
    #[arg(short, long)]
    pub verbose: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    print_banner();

    let config = Config {
        target:  cli.target,
        api_key: cli.key,
        model:   cli.model,
        verbose: cli.verbose,
    };

    session::run(config, &cli.output, cli.auto).await;
}

fn print_banner() {
    println!("{}", r#"
  ██████╗ ██████╗  ██████╗ ███╗   ███╗██████╗ ████████╗    ██╗     ███████╗ █████╗ ██╗  ██╗
  ██╔══██╗██╔══██╗██╔═══██╗████╗ ████║██╔══██╗╚══██╔══╝    ██║     ██╔════╝██╔══██╗██║ ██╔╝
  ██████╔╝██████╔╝██║   ██║██╔████╔██║██████╔╝   ██║       ██║     █████╗  ███████║█████╔╝
  ██╔═══╝ ██╔══██╗██║   ██║██║╚██╔╝██║██╔═══╝    ██║       ██║     ██╔══╝  ██╔══██║██╔═██╗
  ██║     ██║  ██║╚██████╔╝██║ ╚═╝ ██║██║        ██║       ███████╗███████╗██║  ██║██║  ██╗
  ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝╚═╝        ╚═╝       ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝
    "#.red().bold());
    println!("{}", "  System Prompt Exfiltration Agent  ·  Kali Linux  ·  LLM Red Team\n".dimmed());
    println!("{}", "  ⚠  Pour tests autorisés uniquement.\n".yellow());
}
