use crate::crypto::{self, KEY_LEN};
use sha2::{Digest, Sha256};

pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub webhook_secret: String,
    pub master_key: [u8; KEY_LEN],
    pub student_firstname: String,
    pub student_login: String,
    pub kickoff_token: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;

        let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let server_port = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|_| "SERVER_PORT must be a valid port number")?;

        let webhook_secret =
            std::env::var("WEBHOOK_SECRET").unwrap_or_else(|_| "dev-webhook-secret".to_string());

        let master_key_hex = std::env::var("MASTER_KEY_HEX")
            .map_err(|_| "MASTER_KEY_HEX must be set (64 hex chars = 32 bytes)")?;
        let master_key = crypto::parse_key_from_hex(&master_key_hex)?;

        let student_firstname =
            std::env::var("STUDENT_FIRSTNAME").map_err(|_| "STUDENT_FIRSTNAME must be set")?;
        let student_login =
            std::env::var("STUDENT_LOGIN").map_err(|_| "STUDENT_LOGIN must be set")?;
        let kickoff_token = compute_kickoff_token(&student_firstname, &student_login);

        Ok(Self {
            database_url,
            server_host,
            server_port,
            webhook_secret,
            master_key,
            student_firstname,
            student_login,
            kickoff_token,
        })
    }
}

pub fn compute_kickoff_token(firstname: &str, login: &str) -> String {
    let plaintext = format!("{firstname}{login}VIGIL2026");
    let mut hasher = Sha256::new();
    hasher.update(plaintext.as_bytes());
    let digest = hasher.finalize();
    hex::encode(digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kickoff_token_matches_expected_recipe() {
        let token = compute_kickoff_token("test_first", "test_login");

        assert_eq!(
            token,
            "a57331494baa2b4561cc78f74f75400bee1e362979136ff6998a09136ef57e65"
        );
    }

    #[test]
    fn kickoff_token_is_stable() {
        let a = compute_kickoff_token("Alice", "alice.doe");
        let b = compute_kickoff_token("Alice", "alice.doe");
        assert_eq!(a, b);
    }

    #[test]
    fn kickoff_token_different_inputs_differ() {
        let a = compute_kickoff_token("Alice", "alice.doe");
        let b = compute_kickoff_token("Bob", "alice.doe");
        assert_ne!(a, b);
    }
}
