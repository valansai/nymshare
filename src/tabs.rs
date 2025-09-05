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


// installed
use rfd::FileDialog;
use eframe::egui::{
    self, 
    Align, Align2, CentralPanel, Color32, Context, Frame, Layout,
    RichText, Rounding, ScrollArea, Stroke, TopBottomPanel, Ui, Visuals,
};


use chrono::{DateTime, Local};
use uuid::Uuid;
use nymlib::nymsocket::SockAddr;


// std
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::Instant;
use std::time::Duration;


// local 
use crate::app::FileSharingApp;
use crate::shareable::Shareable;
use crate::request::DownLoadRequest;
use crate::theme::Tab;
use crate::helper::time_ago;
use crate::app::VERSION;



/// Renders the share tab UI for the file-sharing application.
pub fn render_share_tab(app: &mut FileSharingApp, ui: &mut egui::Ui) {
    // Drag & Drop support
    let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());
    if !dropped_files.is_empty() {
        let mut added_count = 0;
        for file in dropped_files {
            if let Some(path) = file.path {
                if !app.sharable_files.iter().any(|f| f.path == path) {
                    match Shareable::new(path.clone()) {
                        Ok(s) => {
                            app.sharable_files.push(s);
                            added_count += 1;
                        }
                        Err(e) => {
                            app.set_message(e);
                            return;
                        }
                    }
                    app.download_url.clear();
                }
            }
        }
        if added_count > 0 {
            app.set_message(format!("Added {} file(s) via drag & drop", added_count));
        } else {
            app.set_message("No new files added");
        }
    }

    // Drop-target hint
    let hovering_files = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
    if hovering_files {
        let painter = ui.ctx().layer_painter(eframe::egui::LayerId::new(
            eframe::egui::Order::Foreground,
            eframe::egui::Id::new("file_drop_target"),
        ));
        let rect = ui.ctx().screen_rect();
        painter.rect_stroke(
            rect,
            eframe::egui::CornerRadius::same(0),
            Stroke::new(2.0, Color32::BLACK),
            eframe::egui::StrokeKind::Outside,
        );
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "📂 Drop files here to add",
            eframe::egui::TextStyle::Heading.resolve(ui.style()),
            Color32::BLACK,
        );
    }

    // Top controls
    ui.horizontal(|ui| {
        // Add Files button
        if ui.button("✚ Add Files").on_hover_text("Add new files to share").clicked() {
            let mut added_count = 0;
            if let Some(paths) = rfd::FileDialog::new().pick_files() {
                for path in paths {
                    if !app.sharable_files.iter().any(|f| f.path == path) {
                        match Shareable::new(path) {
                            Ok(s) => {
                                app.sharable_files.push(s);
                                added_count += 1;
                            }
                            Err(e) => {
                                app.set_message(e);
                                return;
                            }
                        }
                        app.download_url.clear();
                    }
                }
            }

            if added_count > 0 {
                app.set_message(format!("Added {} file(s)", added_count));
            } else {
                app.set_message("No new files added");
            }
        }

        // Search bar
        ui.label("🔍");
        Frame::default()
            .stroke(Stroke::new(2.0, Color32::LIGHT_BLUE))
            .rounding(Rounding::same(4))
            .inner_margin(4)
            .show(ui, |ui| {
                ui.add(
                    eframe::egui::TextEdit::singleline(&mut app.search_query)
                        .hint_text("Search in selected files...")
                        .desired_width(250.0),
                )
            });

        if ui.button("❌").on_hover_text("Clear search").clicked() {
            app.search_query.clear();
        }
    });

    ui.separator();
    ui.label("📑 Selected Files:");

    // Hide/Activate controls
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.hide_inactive, "Hide Inactive Files")
            .on_hover_text("Hide files that are not currently active for sharing");

        // Count actionable files
        let activate_count = app.sharable_files.iter().filter(|f| !f.is_active()).count();
        let deactivate_count = app.sharable_files.iter().filter(|f| f.is_active()).count();

        // Enable/disable Activate All
        ui.add_enabled_ui(activate_count > 0, |ui| {
            if ui.button("▶ Activate All").on_hover_text("Activate all files for sharing").clicked() {
                for file in &mut app.sharable_files {
                    if !file.is_active() {
                        file.activate();
                    }
                }
                app.set_message(format!("{} file(s) activated", activate_count));
            }
        });

        // Enable/disable Deactivate All
        ui.add_enabled_ui(deactivate_count > 0, |ui| {
            if ui.button("⏸ Deactivate All").on_hover_text("Deactivate all files from sharing").clicked() {
                for file in &mut app.sharable_files {
                    if file.is_active() {
                        file.deactivate();
                    }
                }
                app.set_message(format!("{} file(s) deactivated", deactivate_count));
            }
        });

        // Show message near hide/activate controls
        if !app.share_message.is_empty() && app.show_share_message() {
            ui.separator();
            ui.label(egui::RichText::new(&app.share_message).color(Color32::BLACK));
        }
    });

    ui.add_space(5.0);

    // File list
    let matching_indices: Vec<usize> = if app.search_query.trim().is_empty() {
        app.sharable_files
            .iter()
            .enumerate()
            .filter(|(_, f)| !app.hide_inactive || f.is_active())
            .map(|(i, _)| i)
            .collect()
    } else {
        let q = app.search_query.to_lowercase();
        app.sharable_files
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                f.file_name().unwrap_or_default().to_lowercase().contains(&q)
                    && (!app.hide_inactive || f.is_active())
            })
            .map(|(i, _)| i)
            .collect()
    };

    if matching_indices.is_empty() {
        ui.label("No matching files found.");
    } else {
        let mut remove_index: Option<usize> = None;
        let mut new_message: Option<String> = None;

        ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            for &i in &matching_indices {
                let file = &mut app.sharable_files[i];
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(format!("Name: {}", file.file_name().unwrap_or("Unknown".into()))).on_hover_text("File name");
                            ui.label(format!("Path: {}", file.path.display())).on_hover_text("Full path");
                            ui.label(format!("Total Advertise: {}", file.advertise)).on_hover_text("Advertise count");
                            ui.label(format!("Total Downloads: {}", file.downloads)).on_hover_text("Downloads count");
                            ui.label(format!("Status: {}", if file.is_active() { "✅ Active" } else { "❌ Inactive" }))
                                .on_hover_text("Active status");
                        });

                        ui.with_layout(
                            eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                            |ui| {
                                let mut style = ui.style().as_ref().clone();
                                style.visuals.widgets.inactive.bg_stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
                                style.visuals.widgets.hovered.bg_stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
                                style.visuals.widgets.active.bg_stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
                                ui.set_style(style);

                                if ui.button("✖ Remove").clicked() {
                                    remove_index = Some(i);
                                    new_message = Some("File removed".to_string());
                                }

                                if ui.button("📋 Copy Link").clicked() {
                                    let link = format!("{}::{}", app.serving_addr, file.file_name().unwrap_or_default());
                                    ui.ctx().output_mut(|out| out.copied_text = link.clone());
                                    new_message = Some("Link copied".to_string());
                                }

                                if file.is_active() {
                                    if ui.button("⏸ Deactivate").clicked() {
                                        file.deactivate();
                                        new_message = Some(format!("Deactivated {}", file.file_name().unwrap_or_default()));
                                    }
                                } else if ui.button("▶ Activate").clicked() {
                                    file.activate();
                                    new_message = Some(format!("Activated {}", file.file_name().unwrap_or_default()));
                                }
                            },
                        );
                    });
                });
                ui.add_space(5.0);
            }
        });

        if let Some(i) = remove_index {
            app.sharable_files.remove(i);
        }

        if let Some(msg) = new_message {
            app.set_message(msg);
        }

        // Show message below file list
        if !app.share_message.is_empty() && app.show_share_message() {
            ui.label(egui::RichText::new(&app.share_message).color(Color32::BLACK));
        }
    }

    // Footer
    eframe::egui::TopBottomPanel::bottom("share_bottom_panel").show(ui.ctx(), |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("NymShare v{}", VERSION));
            ui.separator();
            let active_count = app.sharable_files.iter().filter(|f| f.is_active()).count();
            ui.label(format!("Shareable Files: {} (Active: {})", app.sharable_files.len(), active_count))
                .on_hover_text("Total files / active files");

            if !app.serving_addr.is_empty() {
                ui.separator();
                let max_display_len = 60;
                let display_addr = if app.serving_addr.len() > max_display_len {
                    format!("{}...", &app.serving_addr[..max_display_len - 3])
                } else {
                    app.serving_addr.clone()
                };
                ui.label(format!("Serving at: {}", display_addr)).on_hover_text(&app.serving_addr);
                if ui.button("📋 Copy").clicked() {
                    ui.ctx().output_mut(|out| out.copied_text = app.serving_addr.clone());
                    app.set_message("Serving address copied to clipboard");
                }
            }
        });
    });
}



/// Renders the download tab UI for the file-sharing application.
pub fn render_download_tab(app: &mut FileSharingApp, ui: &mut egui::Ui) {
    // URL input + Download button
    ui.horizontal(|ui| {
        Frame::default()
            .stroke(Stroke::new(2.0, Color32::LIGHT_BLUE))
            .rounding(Rounding::same(4))
            .inner_margin(4.0)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut app.download_url)
                        .desired_width(ui.available_width() - 120.0)
                        .hint_text("🔗 Enter a NymShare service link"),
                );
            });

        // Style for Download button
        let mut style = ui.style().as_ref().clone();
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
        style.visuals.widgets.active.bg_stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
        ui.set_style(style);

        // Download button
        if ui.button("⬇️ Download").clicked() {
            let url = app.download_url.clone();
            app.download_url.clear();
            handle_download_request(app, &url);
        }
    });

    ui.add_space(10.0);

    // Download display options
    ui.label("Download Display Options:");
    ui.horizontal(|ui| {
        // Helper macro for mutually exclusive checkboxes with hover text
        macro_rules! exclusive_checkbox {
            ($field:expr, $other1:expr, $other2:expr, $label:expr, $hover:expr) => {{
                let resp = ui.checkbox(&mut $field, $label).on_hover_text($hover);
                if resp.changed() && $field {
                    $other1 = false;
                    $other2 = false;
                    app.hide_all_downloads = false; // unhide when a filter is selected
                } else if resp.changed() && !$field {
                    $field = false;
                    $other1 = false;
                    $other2 = false;
                    app.show_all_downloads = true; // default to Show All
                }
                resp
            }};
        }

        // Filters
        exclusive_checkbox!(
            app.show_all_downloads,
            app.show_today_downloads,
            app.show_runtime_downloads,
            "Show All",
            "Display all downloads"
        );
        exclusive_checkbox!(
            app.show_today_downloads,
            app.show_all_downloads,
            app.show_runtime_downloads,
            "Show Today's",
            "Show only downloads from today"
        );
        exclusive_checkbox!(
            app.show_runtime_downloads,
            app.show_all_downloads,
            app.show_today_downloads,
            "Show Runtime",
            "Show only downloads since app start"
        );

        // Independent Hide All Downloads checkbox
        ui.checkbox(&mut app.hide_all_downloads, "Hide All")
            .on_hover_text("Hide all download entries")
            .changed()
            .then(|| {
                if app.hide_all_downloads {
                    app.show_all_downloads = false;
                    app.show_today_downloads = false;
                    app.show_runtime_downloads = false;
                } else {
                    app.show_all_downloads = true;
                }
            });
    });

    ui.separator();
    ui.label("📥 Downloaded Files:");

    if app.hide_all_downloads {
        ui.label("Downloads hidden (uncheck 'Hide All' to show).");
        return;
    }

    let now = SystemTime::now();
    let today = Local::now().date_naive();
    let app_start_time = app.start_time.unwrap_or(now);

    // Read all files from the download directory
    let mut download_files: Vec<_> = match fs::read_dir(&app.download_dir) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .map(|entry| entry.path())
            .collect(),
        Err(e) => {
            app.download_message = format!("Failed to read download directory: {}", e);
            Vec::new()
        }
    };

    // Declarative filter closure accepting &PathBuf
    let filter_file = |path_buf: &PathBuf| -> bool {
        let path = path_buf.as_path();
        if app.show_all_downloads {
            return true;
        }
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return false,
        };
        let modified = match metadata.modified() {
            Ok(t) => t,
            Err(_) => return false,
        };
        let file_date = DateTime::<Local>::from(modified).date_naive();

        (app.show_today_downloads && file_date == today)
            || (app.show_runtime_downloads && modified >= app_start_time)
    };

    download_files.retain(filter_file);

    if download_files.is_empty() {
        ui.label("No files match the selected filters.");
    } else {
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            let mut delete_path = None;
            for path in &download_files {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                            ui.label(format!("Path: {}", path.display()));
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("❌ Delete").clicked() {
                                delete_path = Some(path.clone());
                            }
                        });
                    });
                });
                ui.add_space(5.0);
            }

            if let Some(path) = delete_path {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                // Remove the file
                if let Err(e) = fs::remove_file(&path) {
                    app.set_message(format!("Failed to delete file: {}", e));
                } else {
                    app.set_message(format!("Deleted file: {}", file_name));
                    // Remove corresponding download request from app
                    //app.requested_files.retain(|request| request.filename != file_name);
                }
            }
        });
    }

    // Footer
    eframe::egui::TopBottomPanel::bottom("download_bottom_panel").show(ui.ctx(), |ui| {
        ui.horizontal(|ui| {
            // Left: version + download message
            ui.label(format!("NymShare v{}", VERSION));
            ui.separator();
            if !app.download_message.is_empty() && app.show_message() {
                ui.label(RichText::new(&app.download_message).color(Color32::BLACK));
            }

            // Count total downloads
            let total_count = download_files.len();
            ui.label(format!("Total downloads: {}", total_count));

            // Requests button + Settings button
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let mut style = ui.style().as_ref().clone();
                style.visuals.widgets.inactive.bg_stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
                style.visuals.widgets.hovered.bg_stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
                style.visuals.widgets.active.bg_stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
                ui.set_style(style);

                if ui.button("Requests").clicked() {
                    app.active_tab = Tab::DownloadRequests;
                }

                if ui.button("⚙ Settings").clicked() {
                    app.show_download_settings = !app.show_download_settings;
                }

                // Settings window
                if app.show_download_settings {
                    let mut open_flag = app.show_download_settings;
                    egui::Window::new("⚙️ Settings")
                        .open(&mut open_flag)
                        .resizable(false)
                        .collapsible(false)
                        .show(ui.ctx(), |ui| {
                            ui.label(format!(
                                "Current Download Directory: {}",
                                app.download_dir.display()
                            ));

                            if ui.button("📂 Change Download Directory").clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    app.download_dir = path;
                                    app.set_message(format!(
                                        "Download directory changed to: {}",
                                        app.download_dir.display()
                                    ));
                                } else {
                                    app.set_message("No directory selected".to_string());
                                }
                            }
                        });

                    app.show_download_settings = open_flag;
                }
            });
        });
    });
}





/// Renders the download requests tab UI for the file-sharing application.
pub fn render_download_requests_tab(app: &mut FileSharingApp, ui: &mut egui::Ui) {
    ui.heading("📄 Download Requests");
    ui.separator();

    if app.requested_files.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label("No download requests yet.");
        });
        return;
    }

    // Filters
    ui.horizontal(|ui| {
        macro_rules! exclusive_checkbox {
            ($field:expr, $other1:expr, $other2:expr, $label:expr, $hover:expr) => {{
                let resp = ui.checkbox(&mut $field, $label).on_hover_text($hover);
                if resp.changed() && $field {
                    $other1 = false;
                    $other2 = false;
                    app.hide_all_requests = false;
                } else if resp.changed() && !$field {
                    $field = false;
                    $other1 = false;
                    $other2 = false;
                    app.show_all_requests = true;
                }
                resp
            }};
        }

        exclusive_checkbox!(
            app.show_all_requests,
            app.show_accepted_requests,
            app.show_completed_requests,
            "Show All",
            "Display all requests"
        );
        exclusive_checkbox!(
            app.show_accepted_requests,
            app.show_all_requests,
            app.show_completed_requests,
            "Show Accepted",
            "Show only accepted requests"
        );
        exclusive_checkbox!(
            app.show_completed_requests,
            app.show_all_requests,
            app.show_accepted_requests,
            "Show Completed",
            "Show only completed requests"
        );

        // Hide All Requests (independent)
        ui.checkbox(&mut app.hide_all_requests, "Hide All")
            .on_hover_text("Hide all requests")
            .changed()
            .then(|| {
                if app.hide_all_requests {
                    app.show_all_requests = false;
                    app.show_accepted_requests = false;
                    app.show_completed_requests = false;
                } else {
                    app.show_all_requests = true;
                }
            });
    });

    ui.separator();

    if app.hide_all_requests {
        ui.label("Requests hidden (uncheck 'Hide All' to show).");
        return;
    }

    // Filtered requests
    let filtered_requests: Vec<_> = app
        .requested_files
        .iter_mut() 
        .filter(|r| {
            if app.show_all_requests {
                true
            } else if app.show_accepted_requests {
                r.accepted
            } else if app.show_completed_requests {
                r.completed
            } else {
                true
            }
        })
        .collect();

    if filtered_requests.is_empty() {
        ui.label("No requests match the selected filters.");
        return;
    }

    // Scrollable request frames
    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for req in filtered_requests {
                Frame::group(ui.style())
                    .fill(Color32::from_gray(245))
                    .corner_radius(6.0) 
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // request info
                            ui.vertical(|ui| {
                                ui.label(format!("Filename: {}", req.filename))
                                    .on_hover_text("Name of the requested file");
                                ui.label(format!(
                                    "Status: {}",
                                    if req.sent { "✅ Sent" } else { "⏳ Pending" }
                                ))
                                .on_hover_text("Request status");

                                if let Some(sent_time) = req.sent_time {
                                    ui.label(format!("Sent: {}", time_ago(sent_time)))
                                        .on_hover_text("Time since the request was sent");
                                    ui.label(format!(
                                        "Accepted: {}",
                                        if req.accepted { "✅" } else { "⏳ Pending" }
                                    ))
                                    .on_hover_text("Whether the request has been accepted");
                                    ui.label(format!(
                                        "Completed: {}",
                                        if req.completed { "✅" } else { "⏳ Pending" }
                                    ))
                                    .on_hover_text("Whether the request has been completed");
                                }
                            });

                            // buttons
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let mut style = ui.style().as_ref().clone();
                                style.visuals.widgets.inactive.bg_stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
                                style.visuals.widgets.hovered.bg_stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
                                style.visuals.widgets.active.bg_stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
                                ui.set_style(style);

                                let (resend_enabled, hover_msg) = if !req.sent {
                                    (false, "Cannot resend: Request not yet sent")
                                } else if req.accepted {
                                    (false, "Cannot resend: Request already accepted")
                                } else if let Some(sent_time) = req.sent_time {
                                    if sent_time.elapsed() < Duration::from_secs(60) {
                                        (false, "Cannot resend: Wait 1 minute before resending")
                                    } else {
                                        (true, "Resend the request")
                                    }
                                } else {
                                    (false, "Cannot resend: Unknown state")
                                };

                                ui.add_enabled(resend_enabled, egui::Button::new("🔁 Resend"))
                                    .on_hover_text(hover_msg)
                                    .on_disabled_hover_text(hover_msg)
                                    .clicked()
                                    .then(|| {
                                        req.sent = false;
                                        req.sent_time = None;
                                    });
                            });
                        });
                    });
                ui.add_space(4.0);
            }
        });

    // Footer
    eframe::egui::TopBottomPanel::bottom("requests_bottom_panel").show(ui.ctx(), |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("NymShare v{}", VERSION));
            ui.separator();
            let total = app.requested_files.len();
            let accepted = app.requested_files.iter().filter(|r| r.accepted).count();
            let completed = app.requested_files.iter().filter(|r| r.completed).count();
            ui.label(format!(
                "Total Requests: {} | Accepted: {} | Completed: {}",
                total, accepted, completed
            ));
        });
    });
}



/// Handles adding a new download request.
///
/// Splits the provided URL into service address and filename, validates it,
/// prevents duplicates, and pushes a new Requests into the app state.
///
/// Arguments:
/// - app: mutable reference to FileSharingApp
/// - url: the download URL, in the format service::filename
pub fn handle_download_request(app: &mut FileSharingApp, url: &str) {
    // Ignore empty input
    if url.trim().is_empty() {
        app.set_popup_message("Please enter a URL");
        return;
    }

    // Split URL into service address and filename
    let parts: Vec<&str> = url.split("::").collect();

    // Ensure valid format
    if parts.len() != 2 {
        app.set_popup_message("Invalid URL format. Use service::filename");
        return;
    }

    // Service address
    let service_addr = parts[0].to_string();
    // Requested filename
    let filename = parts[1].to_string();

    // Generate unique request ID
    let request_id = Uuid::new_v4().to_string();

    // Convert service address to SockAddr
    let sock_addr = SockAddr::from(service_addr.as_str());

    // Check for duplicate requests
    let already_requested = app.requested_files.iter().any(|r| {
        r.filename == filename && r.from == sock_addr
    });

    if already_requested {
        app.set_message(format!("Download request for '{}' from this service already exists", filename));
        return;
    }

    // Create and push new request
    let mut request = DownLoadRequest::new(sock_addr, filename.clone(), request_id);
    app.requested_files.push(request);
    app.set_message(format!("Download request added: {}", filename));
}