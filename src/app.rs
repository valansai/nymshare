// MIT License
// Copyright (c) Valan Sai 2025
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions.
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


// External crates

use nymlib::nymsocket::SocketMode;
use paste::paste;
use eframe::egui::{self, CentralPanel, Context, TopBottomPanel, Ui, Visuals};

// Standard library
use std::path::PathBuf;
use std::time::{SystemTime, Instant};
use std::collections::HashSet;

// local
use crate::theme::{Theme, Tab};
use crate::tabs::{render_share_tab, render_download_tab, render_explore_tab};
use crate::shareable::Shareable;
use crate::define_tab_messages;
use crate::timed_message;
use crate::define_generic_messages;
use crate::request::{DownLoadRequest, ExploreRequest};


pub static VERSION: &str = "0.0.2";


#[derive(Clone)]
pub enum AppUpdate {
             
}

#[derive(Clone)]
pub struct FileSharingApp {
    // Core application state
    pub start_time: Option<SystemTime>,         // Tracks when the application started
    pub active_tab: Tab,                        // Currently active UI tab (Share, Download, etc.)
    pub theme: Theme,                           // UI theme (Light or Dark)
    pub serving_addr: String,                   // Local nym address for file sharing
    pub download_socket_mode: SocketMode,       // Track the download socket mode
    pub advertise_mode: bool,                   // Controls whether files are advertised
    pub debug_logging: bool,                    // Controls whether debug logging is enabled
    pub show_settings_sidebar: bool,            // Show settings sidebar

    // Share Tab state
    pub shareable_files: Vec<Shareable>,        // Files available for sharing
    pub share_message: String,                  // Message displayed in Share tab
    pub share_message_time: Option<Instant>,    // Timestamp for share message
    pub share_popup_message: String,            // Popup message for Share
    pub share_popup_message_time: Option<Instant>, // Popup timestamp
    pub hide_inactive: bool,                    // Hide inactive files in Share tab
    pub show_share_settings_sidebar: bool,      // Show settings sidebar in Share tab

    // Download Tab state
    pub download_dir: PathBuf,                  // Directory for saving downloads
    pub requested_files: Vec<DownLoadRequest>,  // Pending download requests
    pub download_message: String,               // Message displayed in Download tab
    pub download_message_time: Option<Instant>, // Timestamp for download message
    pub download_popup_message: String,         // Popup message for Download
    pub download_popup_message_time: Option<Instant>, // Popup timestamp
    pub show_all_downloads: bool,               // Show all downloads
    pub show_today_downloads: bool,             // Show only today's downloads
    pub show_runtime_downloads: bool,           // Show only downloads since app start
    pub hide_all_downloads: bool,               // Hide all downloads
    pub search_query: String,                   // Filter files in Download tab
    pub download_url: String,                   // URL input for file downloads
    pub show_download_settings: bool,           // Show download settings
    pub show_download_requests_sidebar: bool,   // Show download requests sidebar

    // Download Requests Tab state
    pub download_requests_message: String,      // Message for DownloadRequests tab
    pub download_requests_message_time: Option<Instant>, // Timestamp for DownloadRequests message
    pub download_requests_popup_message: String, // Popup message for DownloadRequests
    pub download_requests_popup_message_time: Option<Instant>, // Popup timestamp
    pub show_all_requests: bool,                // Show all requests
    pub show_accepted_requests: bool,           // Show only accepted requests
    pub show_completed_requests: bool,          // Show only completed requests
    pub hide_all_requests: bool,                // Hide all requests

    // Explorer Tab state
    pub explore_address: String,                // Remote peer address to explore
    pub explore_requests: Vec<ExploreRequest>,  // Pending explore requests
    pub explore_message: String,                // Message displayed in Explorer tab
    pub explore_message_time: Option<Instant>,  // Timestamp for explorer message
    pub explore_popup_message: String,          // Popup message for Explorer
    pub explore_popup_message_time: Option<Instant>, // Popup timestamp
    pub explore_search_query: String,           // Filter requests in Explorer tab
    pub hide_all_explore_requests: bool,        // Hide all explore requests
    pub show_all_explore_requests: bool,        // Show all explore requests
    pub show_accepted_explore_requests: bool,   // Show only accepted explore requests
    pub expanded_requests: HashSet<String>,     // IDs of explore requests with expanded file lists
}

impl Default for FileSharingApp {
    fn default() -> Self {
        Self {
            // Core application state
            start_time: Some(SystemTime::now()),    // Current system time
            active_tab: Tab::Share,                 // Default to Share tab
            theme: Theme::Dark,                     // Default to Dark theme
            serving_addr: String::new(),            // Empty server address
            download_socket_mode: SocketMode::Anonymous, // Default to Anonymous mode
            advertise_mode: false,                  // Default: advertise mode off
            debug_logging: false,                   // Default: debug logging off
            show_settings_sidebar: false,           // Hide settings sidebar

            // Share Tab state
            shareable_files: Vec::new(),            // No shareable files
            share_message: String::new(),           // Empty share message
            share_message_time: None,               // No share message timestamp
            share_popup_message: String::new(),     // Empty share popup message
            share_popup_message_time: None,         // No share popup timestamp
            hide_inactive: false,                   // Show all files by default
            show_share_settings_sidebar: false,     // Hide settings sidebar in Share tab

            // Download Tab state
            download_dir: {
                let dir = PathBuf::from("downloads");
                std::fs::create_dir_all(&dir).expect("Failed to create default download directory");
                dir
            },
            requested_files: Vec::new(),            // Empty download requests
            download_message: String::new(),        // Empty download message
            download_message_time: None,            // No download message timestamp
            download_popup_message: String::new(),  // Empty download popup message
            download_popup_message_time: None,      // No download popup timestamp
            show_all_downloads: true,               // Show all downloads
            show_today_downloads: false,            // Don't filter by today
            show_runtime_downloads: false,          // Don't filter by runtime
            hide_all_downloads: false,              // Don't hide downloads
            search_query: String::new(),            // Empty search query
            download_url: String::new(),            // Empty download URL
            show_download_settings: false,          // Hide download settings
            show_download_requests_sidebar: false,  // Hide requests sidebar

            // Download Requests Tab state
            download_requests_message: String::new(), // Empty DownloadRequests message
            download_requests_message_time: None,   // No DownloadRequests message timestamp
            download_requests_popup_message: String::new(), // Empty DownloadRequests popup message
            download_requests_popup_message_time: None, // No DownloadRequests popup timestamp
            show_all_requests: true,                // Show all requests
            show_accepted_requests: false,          // Hide accepted filter
            show_completed_requests: false,         // Hide completed filter
            hide_all_requests: false,               // Don't hide requests

            // Explorer Tab state
            explore_address: String::new(),         // Empty peer address
            explore_requests: Vec::new(),           // No explore requests
            explore_message: String::new(),         // Empty explorer message
            explore_message_time: None,             // No explorer message timestamp
            explore_popup_message: String::new(),   // Empty explorer popup message
            explore_popup_message_time: None,       // No explorer popup timestamp
            explore_search_query: String::new(),    // Empty explorer search query
            hide_all_explore_requests: false,       // Don't hide requests
            show_all_explore_requests: true,        // Show all requests
            show_accepted_explore_requests: false,  // Hide accepted requests filter
            expanded_requests: HashSet::new(),      // Empty set for expanded request IDs
        }
    }
}

impl FileSharingApp {
    define_tab_messages!(share, 3.0, 5.0);
    define_tab_messages!(download, 3.0, 5.0);
    define_tab_messages!(explore, 3.0, 5.0);
}

impl eframe::App for FileSharingApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let previous_tab = self.active_tab.clone();
        // Apply theme
        ctx.set_visuals(match self.theme {
            Theme::Light => Visuals::light(),
            Theme::Dark => Visuals::dark(),
        });

        // Top navigation panel
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("📂 NymShare");
                ui.separator();

                if ui.selectable_label(self.active_tab == Tab::Share, "📤 Share").clicked() {
                    self.active_tab = Tab::Share;
                }
                if ui.selectable_label(self.active_tab == Tab::Download, "📥 Download").clicked() {
                    self.active_tab = Tab::Download;
                }

                if ui.selectable_label(self.active_tab == Tab::Explore, "🔎 Explore").clicked() {
                    self.active_tab = Tab::Explore;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(match self.theme {
                            Theme::Light => "🌙 Dark Mode",
                            Theme::Dark => "☀️ Light Mode",
                        })
                        .clicked()
                    {
                        self.theme = match self.theme {
                            Theme::Light => Theme::Dark,
                            Theme::Dark => Theme::Light,
                        };
                        ctx.set_visuals(match self.theme {
                            Theme::Light => Visuals::light(),
                            Theme::Dark => Visuals::dark(),
                        });
                    }
                });
            });
        });


        // Close all sidebars if the tab has changed
        if self.active_tab != previous_tab {
            self.show_share_settings_sidebar = false;
            self.show_download_requests_sidebar = false;
            self.show_settings_sidebar = false;
        }


        // Main content panel
        CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            match self.active_tab {
                Tab::Share => render_share_tab(self, ui),
                Tab::Download => render_download_tab(self, ui),
                Tab::Explore => render_explore_tab(self, ui), 
            }
        });

        self.render_share_popup(ctx);
        self.render_download_popup(ctx);
        self.render_explore_popup(ctx);


        ctx.request_repaint();
    }
}

define_generic_messages!(
    (Share, share),
    (Download, download),
    (Explore, explore)
);