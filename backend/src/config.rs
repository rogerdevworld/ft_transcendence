use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub app_env: String,
    pub database_url: String,
    pub redis_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let _ = dotenvy::from_filename("../.env.example").or_else(|_| dotenvy::dotenv());

        let host = env::var("SERVER_HOST").unwrap_or_else(|_| String::from("0.0.0.0"));
        let port_raw = env::var("SERVER_PORT").unwrap_or_else(|_| String::from("3000"));
        let port = port_raw
            .parse::<u16>()
            .map_err(|_| format!("SERVER_PORT inválido: '{}'", port_raw))?;

        let app_env = env::var("APP_ENV").unwrap_or_else(|_| String::from("development"));
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| String::from("Not found environment variable: DATABASE_URL"))?;

        let redis_url = env::var("REDIS_URL")
            .map_err(|_| String::from("Not found environment variable: REDIS_URL"))?;
        tracing::debug!("DATABASE_URL cargada correctamente");

        Ok(Self {
            host,
            port,
            app_env,
            database_url,
            redis_url,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}