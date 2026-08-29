use std::time::Duration;

use skin_core::protocol_bridge::{self, Config};

fn main() {
    match parse_args().and_then(|config| protocol_bridge::run(&config, |line| eprintln!("{line}")))
    {
        Ok(stats) => {
            eprintln!(
                "intercepted={} accepted={} completed={} chunks={} first_delta_ms={}",
                stats.intercepted,
                stats.accepted,
                stats.completed,
                stats.chunks,
                stats
                    .first_delta_ms
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".into())
            );
        }
        Err(error) => {
            eprintln!("protocol bridge failed: {error}");
            std::process::exit(1);
        }
    }
}

fn parse_args() -> Result<Config, String> {
    let mut config = Config::default();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--upstream" => config.upstream = Some(value(&args, &mut index)?.to_owned()),
            "--model" => config.model = value(&args, &mut index)?.to_owned(),
            "--api-key-env" => config.api_key_env = value(&args, &mut index)?.to_owned(),
            "--port" => {
                config.bridge_port = value(&args, &mut index)?
                    .parse()
                    .map_err(|_| "--port must be a valid u16".to_string())?
            }
            "--cdp-port" => {
                config.cdp_port = value(&args, &mut index)?
                    .parse()
                    .map_err(|_| "--cdp-port must be a valid u16".to_string())?
            }
            "--only-prompt" => config.only_prompt = Some(value(&args, &mut index)?.to_owned()),
            "--ttl" => {
                let seconds = value(&args, &mut index)?
                    .parse::<u64>()
                    .map_err(|_| "--ttl must be an integer number of seconds".to_string())?;
                config.ttl = Duration::from_secs(seconds);
            }
            "--mock-delay" => {
                let seconds = value(&args, &mut index)?
                    .parse::<f64>()
                    .map_err(|_| "--mock-delay must be a number of seconds".to_string())?;
                if !seconds.is_finite() || seconds < 0.0 {
                    return Err("--mock-delay must not be negative".into());
                }
                config.mock_delay = Duration::from_secs_f64(seconds);
            }
            "--once" => config.once = true,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
        index += 1;
    }
    Ok(config)
}

fn value<'a>(args: &'a [String], index: &mut usize) -> Result<&'a str, String> {
    *index += 1;
    args.get(*index)
        .map(String::as_str)
        .ok_or_else(|| "missing value for the previous argument".to_string())
}

fn print_help() {
    println!(
        "Usage: protocol-bridge [options]\n\
         \n\
         Options:\n\
           --upstream URL       OpenAI base URL or full chat/completions URL\n\
           --model NAME         Upstream model name (default: mock)\n\
           --api-key-env NAME   Environment variable containing the API key\n\
           --port PORT          Loopback bridge port (default: 18766)\n\
           --cdp-port PORT      DoubaoWork CDP port (default: 9222)\n\
           --only-prompt TEXT   Intercept only an exact plain-text prompt\n\
           --once               Exit after the intercepted response completes\n\
           --ttl SECONDS        Maximum runtime, 15-3600 seconds\n\
           --mock-delay SEC     Delay between mock deltas\n"
    );
}
