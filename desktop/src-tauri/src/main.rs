#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::api::process::{Command, CommandChild};
use tauri::Manager;

struct BackendProcess(Mutex<Option<CommandChild>>);

fn wait_backend_ready() -> bool {
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(800))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    for _ in 0..120 {
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
            let mut envs = HashMap::new();
            envs.insert("BACKEND_PORT".to_string(), "5461".to_string());
            if let Some(resource_dir) = app.path_resolver().resource_dir() {
                envs.insert(
                    "BANANA_FRONTEND_DIST".to_string(),
                    resource_dir.join("frontend").join("dist").to_string_lossy().to_string(),
                );
            }
            envs.insert(
                "CORS_ORIGINS".to_string(),
                [
                    "tauri://localhost",
                    "http://tauri.localhost",
                    "https://tauri.localhost",
                    "http://localhost:1420",
                    "http://127.0.0.1:5461",
                ]
                .join(","),
            );
            envs.insert("FLASK_ENV".to_string(), "production".to_string());
            sidecar = sidecar.envs(envs);

            let (_rx, child) = sidecar.spawn().expect("failed to spawn backend sidecar");
            app.manage(BackendProcess(Mutex::new(Some(child))));

            let _ = wait_backend_ready();

            tauri::WindowBuilder::new(
                app,
                "main",
                tauri::WindowUrl::App("index.html".into()),
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
