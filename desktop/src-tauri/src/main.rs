#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::thread;
use std::time::Duration;
use tauri::api::process::Command;

fn wait_backend_ready() -> bool {
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(800))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    for _ in 0..40 {
        if let Ok(resp) = client.get("http://127.0.0.1:5461/health").send() {
            if resp.status().is_success() {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(300));
    }
    false
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let mut sidecar = Command::new_sidecar("banana-backend")
                .expect("failed to create sidecar command");
            sidecar = sidecar
                .env("BACKEND_PORT", "5461")
                .env("CORS_ORIGINS", "tauri://localhost,http://tauri.localhost")
                .env("FLASK_ENV", "production");

            let (_rx, _child) = sidecar.spawn().expect("failed to spawn backend sidecar");

            let _ = wait_backend_ready();

            tauri::WindowBuilder::new(
                app,
                "main",
                tauri::WindowUrl::External(
                    "http://127.0.0.1:5461"
                        .parse()
                        .expect("invalid desktop backend URL"),
                ),
            )
            .title("Banana Slides")
            .inner_size(1366.0, 860.0)
            .min_inner_size(1000.0, 700.0)
            .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
