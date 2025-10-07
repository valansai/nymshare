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
use simplelog::*;

// Standard library
use std::time::Instant;
use std::fs::OpenOptions;

/// Initializes logging to a file.
pub fn init_logging(log_file_path: &str) {
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(log_file_path)
        .expect("Unable to open log file");

    let config = ConfigBuilder::new()
        .set_max_level(LevelFilter::Off)
        .add_filter_allow_str("NymShare")
        .build();

    WriteLogger::init(LevelFilter::Debug, config, log_file)
        .expect("Failed to initialize logger");
}

/// Converts elapsed time since sent_time to a human readable format.
pub fn time_ago(sent_time: Instant) -> String {
    let elapsed = sent_time.elapsed();
    if elapsed.as_secs() < 60 {
        format!("{} seconds ago", elapsed.as_secs())
    } else if elapsed.as_secs() < 3600 {
        format!("{} minutes ago", elapsed.as_secs() / 60)
    } else if elapsed.as_secs() < 86400 {
        format!("{} hours ago", elapsed.as_secs() / 3600)
    } else {
        format!("{} days ago", elapsed.as_secs() / 86400)
    }
}

/// Converts bytes to a string of hexadecimal characters.
#[macro_export]
macro_rules! to_hex {
    ($data:expr) => {{
        let bytes: &[u8] = &$data; 
        let mut hex_string = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            hex_string.push_str(&format!("{:02x}", byte));
        }
        hex_string
    }};
}


/// Converts a file size in bytes into a human-readable string.
pub fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}