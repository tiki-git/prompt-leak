// attack/session.rs — boucle semi-autonome de l'agent

use colored::*;
use std::io::{self, Write};

use crate::attack::{client::LlmClient, evaluator, payloads};
use crate::config::Config;
use crate::report;

/// Résultat d'une tentative stocké pour le rapport
pub struct Attempt {
    pub technique_name: String,
    pub prompt:         String,
    pub response:       String,
    pub score:          f32,
    pub label:          String,
    pub fragments:      Vec<String>,
}

pub async fn run(config: Config, output_path: &str, auto: bool) {

    // ── En-tête de session ────────────────────────────────────────────────────
    println!(
        "\n{}  {}\n{}  {}\n{}  {}\n",
        "►  Cible  :".bold(), config.target.cyan(),
        "►  Modèle :".bold(), config.model.cyan(),
        "►  Rapport:".bold(), output_path.cyan(),
    );

    let llm = LlmClient::new(config.clone());

    // ── Vérification de connectivité ──────────────────────────────────────────
    print!("{}", "  Connexion à la cible... ".dimmed());
    io::stdout().flush().unwrap();

    if !llm.probe().await {
        println!("{}", "ECHEC".red().bold());
        println!("  Impossible de joindre {}.", config.target);
        println!("  Vérifie l'URL et qu'Ollama tourne : ollama serve");
        return;
    }
    println!("{}", "OK".green().bold());

    let techniques  = payloads::all();
    let mut attempts: Vec<Attempt> = vec![];
    let mut best_score: f32 = 0.0;
    let mut successful_prompts: Vec<String> = vec![];

    println!(
        "\n{}\n  {} techniques — niveaux L1 (subtil) → L3 (agressif)\n{}",
        "══════════════════════════════════════════".dimmed(),
        techniques.len(),
        "══════════════════════════════════════════".dimmed(),
    );

    // ── Phase 1 : toutes les techniques de base ───────────────────────────────
    for (i, tech) in techniques.iter().enumerate() {

        // Couleur selon le niveau d'agressivité
        let level_label = match tech.level {
            1 => format!("[L{}]", tech.level).green(),
            2 => format!("[L{}]", tech.level).yellow(),
            _ => format!("[L{}]", tech.level).red(),
        };

        // Confirmation semi-autonome
        if !auto {
            print!(
                "\n  {} [{:02}/{}]  {}\n      {}\n  Lancer ? [O/n] ",
                level_label,
                i + 1,
                techniques.len(),
                tech.name.bold(),
                tech.prompt.chars().take(60).collect::<String>().dimmed(),
            );
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let ans = input.trim().to_lowercase();

            if ans == "n" || ans == "non" {
                println!("  {} Ignoré.", "↷".yellow());
                continue;
            }
        } else {
            println!(
                "\n  {} [{:02}/{}]  {}",
                level_label,
                i + 1,
                techniques.len(),
                tech.name.bold(),
            );
        }

        // Envoi du prompt
        match llm.chat(tech.prompt).await {
            Err(e) => {
                println!("    {} {}", "✗".red(), e);
            }
            Ok(response) => {
                let eval  = evaluator::evaluate(&response);
                let lbl   = evaluator::label(eval.score);

                // Icône selon le score
                let icon = if eval.score >= 0.5 {
                    "⚡".yellow().bold().to_string()
                } else if eval.score >= 0.25 {
                    "▲".yellow().to_string()
                } else {
                    "·".dimmed().to_string()
                };

                println!(
                    "    {}  score={:.2}  {}  — {}",
                    icon,
                    eval.score,
                    lbl.bold(),
                    eval.reason.dimmed(),
                );

                // Affichage verbose
                if config.verbose {
                    println!("    {}", "── réponse ──".dimmed());
                    for line in response.lines().take(5) {
                        println!("    {}", line.dimmed());
                    }
                }

                // Mise à jour du meilleur score
                if eval.score > best_score {
                    best_score = eval.score;
                }

                // Stocke les prompts partiellement réussis pour mutation
                if eval.score >= 0.25 {
                    successful_prompts.push(tech.prompt.to_string());
                }

                attempts.push(Attempt {
                    technique_name: tech.name.to_string(),
                    prompt:         tech.prompt.to_string(),
                    response:       response.clone(),
                    score:          eval.score,
                    label:          lbl.to_string(),
                    fragments:      eval.fragments,
                });
            }
        }
    }

    // ── Phase 2 : mutations des prompts réussis ───────────────────────────────
    if !successful_prompts.is_empty() {
        println!(
            "\n{}\n  {} Round de mutation — {} prompt(s) à muter\n{}",
            "══════════════════════════════════════════".dimmed(),
            "⚡".yellow().bold(),
            successful_prompts.len(),
            "══════════════════════════════════════════".dimmed(),
        );

        for base in &successful_prompts {
            let mutations = payloads::mutate(base, 3);

            for (j, mutated) in mutations.iter().enumerate() {
                if !auto {
                    print!(
                        "\n  [M{:02}]  {}\n  Lancer ? [O/n] ",
                        j + 1,
                        mutated.chars().take(65).collect::<String>().dimmed(),
                    );
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    if input.trim().to_lowercase() == "n" {
                        continue;
                    }
                } else {
                    println!(
                        "\n  [M{:02}]  {}",
                        j + 1,
                        mutated.chars().take(65).collect::<String>().dimmed(),
                    );
                }

                if let Ok(response) = llm.chat(mutated).await {
                    let eval = evaluator::evaluate(&response);
                    let lbl  = evaluator::label(eval.score);

                    println!(
                        "    {}  score={:.2}  {}",
                        if eval.score >= 0.5 { "⚡".yellow().bold().to_string() }
                        else { "·".dimmed().to_string() },
                        eval.score,
                        lbl.bold(),
                    );

                    if eval.score > best_score {
                        best_score = eval.score;
                    }

                    attempts.push(Attempt {
                        technique_name: format!("Mutation [{}]", j + 1),
                        prompt:         mutated.clone(),
                        response,
                        score:          eval.score,
                        label:          lbl.to_string(),
                        fragments:      eval.fragments,
                    });
                }
            }
        }
    }

    // ── Résumé final ──────────────────────────────────────────────────────────
    println!(
        "\n{}\n  Meilleur score : {}  ({})\n  Tentatives     : {}\n{}",
        "══════════════════════════════════════════".dimmed(),
        format!("{:.2}", best_score).bold(),
        evaluator::label(best_score).bold(),
        attempts.len(),
        "══════════════════════════════════════════".dimmed(),
    );

    // ── Génération du rapport ─────────────────────────────────────────────────
    report::generate(&config, &attempts, output_path, best_score);
    println!(
        "\n  {}  Rapport sauvegardé → {}\n",
        "✓".green().bold(),
        output_path.cyan(),
    );
}
