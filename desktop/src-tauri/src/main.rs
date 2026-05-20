#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::api::process::{Command, CommandChild, CommandEvent};
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

    for _ in 0..360 {
        if let Ok(resp) = client.get("http://127.0.0.1:5461/health").send() {
            if resp.status().is_success() {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
    false
}

fn append_log(log_file: &str, message: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_file) {
        let _ = writeln!(file, "{}", message);
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let mut sidecar = Command::new_sidecar("banana-backend")
                .expect("failed to create sidecar command");
            let mut envs = HashMap::new();
            envs.insert("BACKEND_PORT".to_string(), "5461".to_string());
            envs.insert("FLASK_ENV".to_string(), "production".to_string());

            let app_data_dir = app
                .path_resolver()
                .app_data_dir()
                .unwrap_or_else(|| std::env::temp_dir().join("banana-slides"));
            let _ = std::fs::create_dir_all(&app_data_dir);
            envs.insert(
                "BANANA_APP_DATA_DIR".to_string(),
                app_data_dir.to_string_lossy().to_string(),
            );

            let log_file = app_data_dir.join("backend.log").to_string_lossy().to_string();
            envs.insert("BANANA_LOG_FILE".to_string(), log_file.clone());

            if let Some(resource_dir) = app.path_resolver().resource_dir() {
                for candidate in [
                    resource_dir.join("frontend").join("dist"),
                    resource_dir.join("dist"),
                ] {
                    if candidate.join("index.html").exists() {
                        envs.insert(
                            "BANANA_FRONTEND_DIST".to_string(),
                            candidate.to_string_lossy().to_string(),
                        );
                        break;
                    }
                }
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
            sidecar = sidecar.envs(envs);

            append_log(&log_file, "Starting Banana Slides backend sidecar...");
            let (mut rx, child) = sidecar.spawn().expect("failed to spawn backend sidecar");
            let sidecar_log_file = log_file.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(event) = rx.recv().await {
                    match event {
                        CommandEvent::Stdout(line) => append_log(&sidecar_log_file, &line),
                        CommandEvent::Stderr(line) => append_log(&sidecar_log_file, &line),
                        CommandEvent::Error(error) => {
                            append_log(&sidecar_log_file, &format!("sidecar error: {}", error));
                        }
                        CommandEvent::Terminated(payload) => {
                            append_log(
                                &sidecar_log_file,
                                &format!("sidecar terminated: {:?}", payload),
                            );
                        }
                        _ => {}
                    }
                }
            });
            app.manage(BackendProcess(Mutex::new(Some(child))));

            if wait_backend_ready() {
                append_log(&log_file, "Backend sidecar is ready.");
            } else {
                append_log(&log_file, "Backend sidecar did not become ready before timeout.");
            }

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
