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
use nymlib::{
    nymsocket::SockAddr,
    serialize::Serialize,
    serialize_derive::impl_serialize_for_struct,
};

// Standard library
use std::time::Instant;

/// Represents a client request to download a file from a remote service.
/// Contains metadata for initiating and tracking a file download.
#[derive(PartialEq, Debug, Clone)]
pub struct DownLoadRequest {
    /// Source service address for the file.
    pub from: SockAddr,

    /// Name of the file to download.
    pub filename: String,

    /// Unique identifier for the request.
    pub request_id: String,

    /// Indicates if the request has been sent.
    pub sent: bool,

    /// Time the request was sent.
    pub sent_time: Option<Instant>,

    /// Time the acknowledgment was received.
    pub ack_time: Option<Instant>,

    /// Indicates if the request was accepted.
    pub accepted: bool,

    /// Indicates if the download is completed.
    pub completed: bool,
}

impl DownLoadRequest {
    /// Creates a new [`DownLoadRequest`] instance.
    ///
    /// The sent field is set to false by default.
    ///
    /// # Arguments
    /// * from - The Nym service address wrapped in SockAddr.
    /// * filename - The target filename to request for download.
    /// * request_id - A unique identifier for tracking this request.
    ///
    /// # Returns
    /// A DownLoadRequest instance initialized with the provided values.
    pub fn new(from: SockAddr, filename: String, request_id: String) -> Self {
        Self {
            from,
            filename,
            request_id,
            sent: false,
            sent_time: None,
            ack_time: None,
            accepted: false,
            completed: false,
        }
    }
}

impl_serialize_for_struct! {
    target DownLoadRequest {
        readwrite(self.request_id);
        readwrite(self.filename);
    }
}
