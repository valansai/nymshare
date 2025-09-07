// MIT License
// Copyright (c) Valan Sai 2025
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

mod app;
mod theme;
mod tabs;
mod shareable;
mod request;
mod helper;
mod network;

#[macro_use]
mod macros;


// External crates
use eframe::{self, egui, App, NativeOptions};
use tokio::sync::{Mutex, mpsc};
use log::{debug, info, warn, error};

// Standard library
use std::sync::Arc;

// local 
use crate::network::initialize_sockets;
use crate::helper::init_logging;
use crate::network::download_manager;
use crate::network::serving_manager;
use crate::app::{FileSharingApp, AppUpdate};





#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // Initialize logging
    init_logging(&"debug.log");

    // Create Tokio runtime for async tasks
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Shared application state
    let app_shared = Arc::new(Mutex::new(FileSharingApp::default()));

    // Initialize sockets
    initialize_sockets(app_shared.clone()).await;

    let app_clone = app_shared.clone();

    // Download manager task
    tokio::spawn({
        let app_clone = app_clone.clone();
        async move {
            if let Err(e) = download_manager(app_clone).await {
                eprintln!("download_manager error: {:?}", e);
            }
        }
    });

    // Serving manager task
    tokio::spawn({
        let app_clone = app_clone.clone();
        async move {
            if let Err(e) = serving_manager(app_clone).await {
                eprintln!("serving_manager error: {:?}", e);
            }
        }
    });

    // Window options
    let options = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([850.0, 400.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    // Wrapper for shared FileSharingApp
    struct AppWrapper {
        app: Arc<Mutex<FileSharingApp>>,
    }

    impl eframe::App for AppWrapper {
        fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            if let Ok(mut app) = self.app.try_lock() {
                FileSharingApp::update(&mut app, ctx, frame);
            } else {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Waiting for app state...");
                });
            }

            ctx.request_repaint();
        }
    }

    // Run native eframe app
    eframe::run_native(
        "NymShare",
        options,
        Box::new(|_cc| Ok(Box::new(AppWrapper { app: app_shared.clone() }) as Box<dyn App>)),
    )
}




