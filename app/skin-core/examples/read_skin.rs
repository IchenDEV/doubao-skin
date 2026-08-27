//! Read data-skin of the app pages via the Rust CDP client (no python).
use std::time::Duration;

fn main() {
    let out = std::process::Command::new("curl")
        .args(["-s", "--max-time", "3", "http://localhost:9222/json"])
        .output()
        .unwrap();
    let targets: Vec<serde_json::Value> = serde_json::from_slice(&out.stdout).unwrap();
    for t in &targets {
        if t.get("type").and_then(|v| v.as_str()) != Some("page") {
            continue;
        }
        let url = t.get("url").and_then(|v| v.as_str()).unwrap_or("");
        if !url.contains("doubaowork") {
            continue;
        }
        let ws_url = t.get("webSocketDebuggerUrl").and_then(|v| v.as_str()).unwrap_or("");
        let r = skin_core::ws::Cdp::connect(ws_url, Duration::from_secs(5))
            .and_then(|mut cdp| {
                let v = cdp.evaluate_with_timeout(
                    "document.documentElement.getAttribute('data-skin')",
                    Duration::from_secs(3),
                );
                cdp.close();
                v
            });
        match r {
            Ok(v) => println!("{url} => {v}"),
            Err(e) => println!("{url} => ERR {e}"),
        }
    }
}
