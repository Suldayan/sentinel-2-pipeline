use dotenvy::dotenv;
use std::env;

fn validate_overview(level: u8) -> Result<u8, String> {
    if (1..=5).contains(&level) {
        Ok(level)
    } else {
        Err(format!("Invalid OVERVIEW_LEVEL={}. Must be between 1 and 5.", level))
    }
}

pub fn load_overview_level() -> u8 {
    dotenv().ok();

    let raw = std::env::var("OVERVIEW_LEVEL").unwrap_or_else(|_| "1".into());

    let parsed: u8 = raw.parse().unwrap_or_else(|_| {
        panic!("OVERVIEW_LEVEL must be a number between 1 and 5, got '{}'", raw);
    });

    validate_overview(parsed).unwrap()
}
