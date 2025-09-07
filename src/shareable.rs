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


// Standard library
use std::fs;
use std::io;
use std::path::PathBuf;

// Represents a file that can be shared
// Holds the file's path, sharing status, and download count
#[derive(Clone)]
pub struct Shareable {
    // The filesystem path to the file
    pub path: PathBuf,

    // True if the file is active and ready for sharing
    pub active: bool,

    // Number of times that we have advertise this file
    pub advertise: u32,

    // Number of times this file has been downloaded
    pub downloads: u32,
}

impl Shareable {
    // Creates a new Shareable instance from a file path
    // Returns an error if the path is invalid, does not exist, or is not a file
    pub fn new(path: PathBuf) -> Result<Self, String> {
        if path.file_name().is_none() {
            return Err("Path must contain a valid file name".to_string());
        }

        if !path.exists() {
            return Err(format!("File does not exist: {:?}", path));
        }

        if !path.is_file() {
            return Err(format!("Path is not a file: {:?}", path));
        }

        Ok(Self {
            path,
            active: false,  // Files start as inactive
            advertise: 0,   // Advertise count startsat 0 
            downloads: 0,   // Download count starts at 0
        })
    }

    // Marks the file as active
    pub fn activate(&mut self) {
        self.active = true;
    }

    // Marks the file as inactive
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    // Returns true if the file is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    // Reads the file contents into a byte vector
    pub fn read_bytes(&self) -> io::Result<Vec<u8>> {
        fs::read(&self.path)
    }

    // Returns the file name as a string if possible
    pub fn file_name(&self) -> Option<String> {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    }
}


