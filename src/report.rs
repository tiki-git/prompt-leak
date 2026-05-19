// report.rs — génération du rapport markdown

use chrono::Local;
use std::fs;

use crate::attack::{evaluator, session::Attempt};
use crate::config::Config;

pub fn generate(
    config:     &Config,
    attempts:   &[Attempt],
    path:       &str,
    best_score: f32,
) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut md = String::new();

    // ── En-tête ───────────────────────────────────────────────────────────────
    md.push_str("# Rapport d'exfiltration de system prompt\n\n");
    md.push_str(&format!("**Cible :** `{}`  \n", config.target));
    md.push_str(&format!("**Modèle :** `{}`  \n", config.model));
    md.push_str(&format!("**Date :** {}  \n\n", now));
    md.push_str("---\n\n");

    // ── Verdict ───────────────────────────────────────────────────────────────
    md.push_str("## Verdict\n\n");
    let verdict = evaluator::label(best_score);
    md.push_str(&format!(
        "**{}** — score maximal : `{:.2}`\n\n",
        verdict, best_score
    ));

    // Statistiques rapides
    let hits     = attempts.iter().filter(|a| a.score >= 0.5).count();
    let partials = attempts.iter().filter(|a| a.score >= 0.25 && a.score < 0.5).count();
    let clean    = attempts.iter().filter(|a| a.score < 0.25).count();

    md.push_str(&format!(
        "| Résultat | Nombre |\n\
         |----------|--------|\n\
         | ⚡ Fuite probable/certaine | {} |\n\
         | ▲ Signal partiel | {} |\n\
         | · Aucun signal | {} |\n\
         | **Total tentatives** | **{}** |\n\n",
        hits, partials, clean, attempts.len()
    ));

    // ── Fragments extraits ────────────────────────────────────────────────────
    let all_fragments: Vec<&String> = attempts
        .iter()
        .flat_map(|a| &a.fragments)
        .collect();

    if !all_fragments.is_empty() {
        md.push_str("## Fragments extraits\n\n");
        md.push_str("> Ces chaînes ont été identifiées comme du contenu probable du system prompt.\n\n");
        for frag in &all_fragments {
            md.push_str(&format!("```\n{}\n```\n\n", frag.trim()));
        }
    }

    // ── Techniques réussies ───────────────────────────────────────────────────
    let hits_list: Vec<&Attempt> = attempts
        .iter()
        .filter(|a| a.score >= 0.5)
        .collect();

    if !hits_list.is_empty() {
        md.push_str("## Techniques réussies\n\n");
        for a in &hits_list {
            md.push_str(&format!(
                "### {} — score `{:.2}`\n\n", a.technique_name, a.score
            ));
            md.push_str(&format!(
                "**Prompt envoyé :**\n```\n{}\n```\n\n", a.prompt
            ));
            md.push_str(&format!(
                "**Réponse obtenue :**\n```\n{}\n```\n\n", a.response.trim()
            ));
            md.push_str("---\n\n");
        }
    }

    // ── Tableau complet ───────────────────────────────────────────────────────
    md.push_str("## Toutes les tentatives\n\n");
    md.push_str("| # | Technique | Score | Verdict |\n");
    md.push_str("|---|-----------|-------|---------|\n");
    for (i, a) in attempts.iter().enumerate() {
        md.push_str(&format!(
            "| {} | {} | `{:.2}` | {} |\n",
            i + 1, a.technique_name, a.score, a.label
        ));
    }
    md.push('\n');

    // ── Recommandations ───────────────────────────────────────────────────────
    md.push_str("## Recommandations\n\n");
    md.push_str("1. **Ne jamais stocker de secrets dans le system prompt** — clés API, URLs internes et credentials doivent être injectés au runtime.\n");
    md.push_str("2. **Traiter le system prompt comme non-secret** — concevoir le prompt en supposant qu'il sera lu par un attaquant.\n");
    md.push_str("3. **Ajouter un guardrail sémantique** — proxy qui détecte et bloque les prompts tentant d'extraire le contexte.\n");
    md.push_str("4. **Monitorer les patterns d'exfiltration** — logger et alerter sur les réponses qui correspondent à la structure du system prompt.\n\n");

    md.push_str("---\n\n_Généré par prompt-leak — pour tests autorisés uniquement._\n");

    // ── Écriture du fichier ───────────────────────────────────────────────────
    match fs::write(path, &md) {
        Ok(_)  => {},
        Err(e) => eprintln!("Erreur écriture rapport : {}", e),
    }
}
