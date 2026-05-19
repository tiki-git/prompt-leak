// config.rs — configuration partagée entre tous les modules

#[derive(Clone, Debug)]
pub struct Config {
    /// URL de base de l'API LLM (ex: http://localhost:11434)
    pub target: String,

    /// Clé API optionnelle (Bearer token)
    pub api_key: Option<String>,

    /// Nom du modèle (ex: llama3, gpt-4, mistral)
    pub model: String,

    /// Affiche les réponses complètes du LLM
    pub verbose: bool,
}

impl Config {
    /// Retourne l'endpoint chat completions complet
    pub fn endpoint(&self) -> String {
        format!(
            "{}/v1/chat/completions",
            self.target.trim_end_matches('/')
        )
    }

    /// Retourne le header Authorization si une clé est fournie
    pub fn auth_header(&self) -> Option<String> {
        self.api_key
            .as_ref()
            .map(|k| format!("Bearer {}", k))
    }
}
