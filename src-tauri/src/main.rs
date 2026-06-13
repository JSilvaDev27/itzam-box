// ItzamBox — Local and open-source alternative to Docker Desktop
// Copyright (C) 2026 SodigTech — GPL-3.0

// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    app_lib::run()
}
