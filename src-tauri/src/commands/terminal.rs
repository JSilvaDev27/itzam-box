// ItzamBox — PTY Terminal Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

pub struct PtySession {
    pub writer: Box<dyn Write + Send>,
    pub child: Option<Box<dyn portable_pty::MasterPty + Send>>,
}

pub struct PtyManager {
    sessions: Mutex<HashMap<String, Arc<Mutex<PtySession>>>>,
    next_id: Mutex<u64>,
}

impl PtyManager {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
pub async fn spawn_host_terminal(
    app: AppHandle,
    pty_state: State<'_, PtyManager>,
) -> Result<String, String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("PTY open error: {}", e))?;

    let shell = if cfg!(target_os = "windows") {
        "powershell.exe"
    } else {
        "/bin/bash"
    };
    let cmd = CommandBuilder::new(shell);
    let _child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("Spawn error: {}", e))?;
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("Clone reader: {}", e))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("Take writer: {}", e))?;

    let id = {
        let mut next = pty_state.next_id.lock().map_err(|e| e.to_string())?;
        let id = format!("pty-{}", *next);
        *next += 1;
        id
    };

    let session = Arc::new(Mutex::new(PtySession {
        writer: Box::new(writer),
        child: None,
    }));
    pty_state
        .sessions
        .lock()
        .map_err(|e| e.to_string())?
        .insert(id.clone(), session);

    // Spawn reader thread
    let app_clone = app.clone();
    let id_clone = id.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app_clone.emit(
                        "pty-output",
                        serde_json::json!({
                            "id": id_clone,
                            "data": data,
                        }),
                    );
                }
                Err(_) => break,
            }
        }
    });

    Ok(id)
}

#[tauri::command]
pub async fn spawn_container_terminal(
    app: AppHandle,
    pty_state: State<'_, PtyManager>,
    container_id: String,
) -> Result<String, String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("PTY error: {}", e))?;

    let mut cmd = CommandBuilder::new("docker");
    cmd.args(["exec", "-it", &container_id, "/bin/sh"]);
    let _child = pair
        .slave
        .spawn_command(cmd)
        .or_else(|_| {
            let mut cmd2 = CommandBuilder::new("docker");
            cmd2.args(["exec", "-it", &container_id, "/bin/bash"]);
            pair.slave.spawn_command(cmd2)
        })
        .map_err(|e| format!("Spawn error: {}", e))?;
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("Reader: {}", e))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("Writer: {}", e))?;

    let id = {
        let mut n = pty_state.next_id.lock().map_err(|e| e.to_string())?;
        let id = format!("pty-{}", *n);
        *n += 1;
        id
    };
    pty_state
        .sessions
        .lock()
        .map_err(|e| e.to_string())?
        .insert(
            id.clone(),
            Arc::new(Mutex::new(PtySession {
                writer: Box::new(writer),
                child: None,
            })),
        );

    let a = app.clone();
    let i = id.clone();
    std::thread::spawn(move || {
        let mut b = [0u8; 4096];
        loop {
            match reader.read(&mut b) {
                Ok(0) => break,
                Ok(n) => {
                    let _ = a.emit(
                        "pty-output",
                        serde_json::json!({"id":i,"data":String::from_utf8_lossy(&b[..n])}),
                    );
                }
                Err(_) => break,
            }
        }
    });
    Ok(id)
}

#[tauri::command]
pub async fn pty_write(
    pty_state: State<'_, PtyManager>,
    id: String,
    data: String,
) -> Result<(), String> {
    let sessions = pty_state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions.get(&id).ok_or("PTY session not found")?;
    let mut s = session.lock().map_err(|e| e.to_string())?;
    s.writer
        .write_all(data.as_bytes())
        .map_err(|e| format!("Write error: {}", e))?;
    s.writer
        .flush()
        .map_err(|e| format!("Flush error: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn pty_resize(
    pty_state: State<'_, PtyManager>,
    id: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    let sessions = pty_state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions.get(&id).ok_or("PTY session not found")?;
    let s = session.lock().map_err(|e| e.to_string())?;
    if let Some(master) = &s.child {
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Resize error: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn pty_close(pty_state: State<'_, PtyManager>, id: String) -> Result<(), String> {
    let mut sessions = pty_state.sessions.lock().map_err(|e| e.to_string())?;
    sessions.remove(&id);
    Ok(())
}
