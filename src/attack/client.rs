// attack/client.rs — client HTTP OpenAI-compatible

use crate::config::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ── Structures de la requête ──────────────────────────────────────────────────

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

// ── Structures de la réponse ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

// ── Client principal ──────────────────────────────────────────────────────────

pub struct LlmClient {
    client: Client,
    config: Config,
}

impl LlmClient {
    /// Crée un nouveau client à partir de la config
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        Self { client, config }
    }

    /// Envoie un prompt et retourne la réponse texte du LLM
    pub async fn chat(&self, prompt: &str) -> Result<String, String> {
        // Construction de la requête
        let body = ChatRequest {
            model: &self.config.model,
            messages: vec![Message {
                role: "user".into(),
                content: prompt.to_string(),
            }],
            temperature: 0.0,
            max_tokens: 1000,
        };

        // Envoi avec auth si nécessaire
        let mut req = self.client
            .post(self.config.endpoint())
            .json(&body);

        if let Some(auth) = self.config.auth_header() {
            req = req.header("Authorization", auth);
        }

        // Traitement de la réponse
        match req.send().await {
            Err(e) => Err(format!("Erreur réseau : {}", e)),
            Ok(resp) => {
                let status = resp.status();
                if !status.is_success() {
                    return Err(format!("HTTP {} — cible inaccessible", status));
                }
                match resp.json::<ChatResponse>().await {
                    Err(e) => Err(format!("Erreur parsing JSON : {}", e)),
                    Ok(r)  => Ok(r.choices
                        .first()
                        .map(|c| c.message.content.clone())
                        .unwrap_or_default()),
                }
            }
        }
    }

    /// Vérifie que l'endpoint répond avant de lancer l'attaque
    pub async fn probe(&self) -> bool {
        self.chat("Hello").await.is_ok()
    }
}
