pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub webhook_secret: String,
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

        Ok(Self {
            database_url,
            server_host,
            server_port,
            webhook_secret,
        })
    }
}
