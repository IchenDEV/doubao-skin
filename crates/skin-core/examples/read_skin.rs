//! Read data-skin from the selected app via the native CDP client.
use std::time::Duration;

fn main() {
    let target = std::env::args()
        .nth(1)
        .as_deref()
        .and_then(skin_core::live::TargetApp::from_id)
        .unwrap_or(skin_core::live::TargetApp::DoubaoWork);
    let out = std::process::Command::new("curl")
        .args([
            "-s",
            "--max-time",
            "3",
            &format!("http://localhost:{}/json", target.port()),
        ])
        .output()
        .unwrap();
    let targets: Vec<serde_json::Value> = serde_json::from_slice(&out.stdout).unwrap();
    for t in &targets {
        if t.get("type").and_then(|v| v.as_str()) != Some("page") {
            continue;
        }
        let url = t.get("url").and_then(|v| v.as_str()).unwrap_or("");
        if !target.matches_identity_url(url) {
            continue;
        }
        let ws_url = t
            .get("webSocketDebuggerUrl")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let r = skin_core::ws::Cdp::connect(ws_url, Duration::from_secs(5)).and_then(|mut cdp| {
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
