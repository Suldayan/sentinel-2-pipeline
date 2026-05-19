use sentinel_orchestrator::run;

fn main() {
    env_logger::init();
    dotenvy::dotenv().ok();

    if let Err(e) = run() {
        log::error!("Fatal error: {e}");
        std::process::exit(1);
    }
}

