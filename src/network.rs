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



// instaled
use nymlib::nymsocket::{Socket, SockAddr, SocketMode};
use nymlib::serialize::{DataStream, Serialize};
use tokio::{
    sync::{broadcast, mpsc, Mutex},
    time::{Duration, interval},
};
use log::{debug, info, warn, error};


// local 
use std::sync::LazyLock;
use std::sync::Arc;
use std::io::Write;
use std::time::Instant;

// local 
use crate::app::FileSharingApp;
use crate::shareable::Shareable;



/// Global reference to the download socket
/// Used to anonymously download files from remote peers
pub static DOWNLOAD_SOCKET: LazyLock<Mutex<Option<Arc<Mutex<Socket>>>>> = 
    LazyLock::new(|| Mutex::new(None));

/// Global reference to the serving socket
/// Used to serve local files to peers in Individual mode
pub static SERVING_SOCKET: LazyLock<Mutex<Option<Arc<Mutex<Socket>>>>> = 
    LazyLock::new(|| Mutex::new(None));

/// Broadcast channel for signaling stop events to background tasks
/// Shared between serving_manager and download_manager
pub static STOP_SIGNAL: LazyLock<Arc<Mutex<Option<broadcast::Sender<bool>>>>> = 
    LazyLock::new(|| Arc::new(Mutex::new(None))); 


/// Initializes both serving and download sockets
/// Spawns background listeners, sets up stop signal, and updates app state
pub async fn initialize_sockets(app: Arc<Mutex<FileSharingApp>>) {
    info!("[*] Started initialize_sockets");

    // initialize download socket (anonymous mode)
    let download_socket = match Socket::new_ephemeral(SocketMode::Anonymous).await {
        Some(s) => s,
        None => {
            error!("Failed to create anonymous socket; aborting");
            return;
        }
    };

    // spawn background listener for download socket
    let mut download_listen_socket = download_socket.clone();
    tokio::spawn(async move {
        download_listen_socket.listen().await;
    });

    let p_socket = Arc::new(Mutex::new(download_socket));
    *DOWNLOAD_SOCKET.lock().await = Some(p_socket.clone());

    // initialize serving socket (individual mode)
    let serving_socket = match Socket::new_standard("serving_datadir", SocketMode::Individual).await {
        Some(s) => s,
        None => {
            error!("Failed to create serving socket; aborting");
            return;
        }
    };

    let serving_socket_addr = serving_socket.getaddr().await;

    // spawn background listener for serving socket
    let mut serving_listen_socket = serving_socket.clone();
    tokio::spawn(async move {
        serving_listen_socket.listen().await;
    });

    let p_socket = Arc::new(Mutex::new(serving_socket));
    *SERVING_SOCKET.lock().await = Some(p_socket.clone());

    // setup stop signal
    let (tx, _rx) = broadcast::channel(1);
    {
        let mut stop_signal = STOP_SIGNAL.lock().await;
        *stop_signal = Some(tx);
    }

    // update app with serving socket address
    {
        let mut app_opt = app.lock().await;
        app_opt.serving_addr = serving_socket_addr
            .expect("Failed to get addr")
            .to_string();
        app_opt.set_message("Socket initialized sucessfully");
    }
}


pub mod COMMANDS {
    pub const FILE_REQUEST: &str = "FILE_REQUEST";   
    pub const GETFILE: &str = "GETFILE";
    pub const ACK_FILE_REQUEST: &str = "ACK_FILE_REQUEST";   
    //pub const ADVERTISE: &str = "ADVERTISE";         
    //pub const GETADVERTISE: &str = "GETADVERTISE";         
}


/// Background task that manages serving local files to peers.
///
/// Responsibilities:
/// 1. Listens for incoming file requests from remote peers.
/// 2. Sends an acknowledgment (ACK) for each valid request.
/// 3. Reads the requested file from disk and sends it to the requester.
/// 4. Updates the app state (download counts, logging).
pub async fn serving_manager(app: Arc<Mutex<FileSharingApp>>) -> Result<(), String> {
    info!("[*] Started serving_manager");

    // Initialize stop signal
    let mut stop_signal_rx = {
        let guard = STOP_SIGNAL.lock().await;
        guard
            .as_ref()
            .ok_or_else(|| "Stop signal not initialized".to_string())?
            .subscribe()
    };

    // Setup periodic interval
    let mut interval = interval(Duration::from_millis(300));

    loop {
        tokio::select! {
            // Handle stop signal
            result = stop_signal_rx.recv() => {
                match result {
                    Ok(true) => {
                        info!("[*] Stopping serving_manager task");
                        break Ok(());
                    }
                    Ok(false) => continue,
                    Err(e) => {
                        info!("[*] Stop signal error: {}", e);
                        break Ok(());
                    }
                }
            }

            // Process incoming messages
            _ = interval.tick() => {
                // Lock socket and drain messages
                let socket_opt = SERVING_SOCKET.lock().await;
                let Some(p_socket) = &*socket_opt else { continue; };
                let mut socket = p_socket.lock().await;

                let messages: Vec<_> = {
                    let mut guard = socket.recv.lock().await;
                    guard.drain(..).collect()
                };

                for message in messages {
                    let mut stream = DataStream::default();
                    stream.write(&message.data);

                    // Extract command
                    let command = match stream.stream_out::<String>() {
                        Ok(cmd) => cmd,
                        Err(_) => {
                            warn!("Invalid message format: missing command");
                            continue;
                        }
                    };

                    // Collect active files
                    let active_files: Vec<Shareable> = {
                        let app_opt = app.lock().await;
                        app_opt.sharable_files.iter().filter(|f| f.is_active()).cloned().collect()
                    };

                    // Handle commands
                    match command.as_str() {
                        COMMANDS::FILE_REQUEST => {
                            if active_files.is_empty() {
                                info!("No active files to serve");
                                continue;
                            }

                            // Extract request info
                            let request_id = match stream.stream_out::<String>() {
                                Ok(id) => id,
                                Err(_) => {
                                    warn!("Missing request_id");
                                    continue;
                                }
                            };
                            let requested_file_name = match stream.stream_out::<String>() {
                                Ok(name) => name,
                                Err(_) => {
                                    warn!("Missing filename");
                                    continue;
                                }
                            };

                            // Serve file if available
                            if let Some(file) = active_files.iter().find(|f|
                                f.file_name().unwrap_or_default() == requested_file_name) {

                                // Send ACK
                                let mut ack_stream = DataStream::default();
                                ack_stream.stream_in(&COMMANDS::ACK_FILE_REQUEST);
                                ack_stream.stream_in(&request_id);
                                if socket.send(ack_stream.data.clone(), message.from.clone()).await {
                                    info!("Sent ACK for '{}' (id={})", requested_file_name, request_id);
                                } else {
                                    warn!("Failed to send ACK for '{}' request", requested_file_name);
                                }

                                // Read file bytes
                                let file_bytes = match file.read_bytes() {
                                    Ok(b) => b,
                                    Err(e) => {
                                        warn!("Failed to read '{}': {:?}", requested_file_name, e);
                                        continue;
                                    }
                                };

                                // Prepare and send file
                                let mut response_stream = DataStream::default();
                                response_stream.stream_in(&COMMANDS::GETFILE);
                                response_stream.stream_in(&request_id);
                                response_stream.stream_in(&file_bytes);

                                if socket.send(response_stream.data.clone(), message.from.clone()).await {
                                    let mut app_opt = app.lock().await;
                                    if let Some(f) = app_opt.sharable_files.iter_mut().find(|f|
                                        f.file_name().unwrap_or_default() == requested_file_name) {
                                        f.downloads += 1;
                                        info!("Replied with file '{}' for request '{}'", requested_file_name, request_id);
                                    }
                                } else {
                                    warn!("Failed to send file '{}'", requested_file_name);
                                }
                            } else {
                                info!("Requested file '{}' not found", requested_file_name);
                            }
                        }

                        // Unknown command
                        _ => {
                            warn!("Received unknown command '{}'", command);
                            continue;
                        }
                    }
                }
            }
        }
    }
}

/// Background task that manages downloads.
///
/// Responsibilities:
/// 1. Periodically sends download requests to peers for files listed in the app state.
/// 2. Receives replies from peers, marking requests as accepted or completed.
/// 3. Writes downloaded file data to the local filesystem.
/// 4. Updates the app state/UI with download progress and completion messages.
pub async fn download_manager(app: Arc<Mutex<FileSharingApp>>) -> Result<(), String> {
    info!("[*] Started download_manager");

    // Initialize stop signal
    let mut stop_signal_rx = {
        let guard = STOP_SIGNAL.lock().await;
        guard.as_ref().ok_or_else(|| "Stop signal not initialized".to_string())?.subscribe()
    };

    // Setup intervals
    let mut send_interval = interval(Duration::from_millis(200));
    let mut process_interval = interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            // Handle stop signal
            result = stop_signal_rx.recv() => {
                match result {
                    Ok(true) => {
                        info!("[*] Stopping download_manager task");
                        break Ok(());
                    }
                    Ok(false) => continue,
                    Err(e) => {
                        info!("[*] Stop signal error: {}", e);
                        break Ok(());
                    }
                }
            }

            // Send pending download requests
            _ = send_interval.tick() => {
                let mut app_opt = app.lock().await;
                let socket_opt = DOWNLOAD_SOCKET.lock().await;
                let Some(p_socket) = &*socket_opt else { continue; };
                let mut download_requests_to_send: Vec<_> = app_opt.requested_files.iter_mut().filter(|r| !r.sent).collect();

                for request in download_requests_to_send {
                    // Prepare request
                    let mut stream = DataStream::default();
                    stream.stream_in(&COMMANDS::FILE_REQUEST);
                    stream.stream_in(request);
                    let serialized = stream.data.clone();

                    let mut socket = p_socket.lock().await;
                    socket.extra_surbs = Some(10);

                    let send_result = socket.send(serialized, request.from.clone()).await;
                    info!("Sent download request to {:?}", request.from);

                    if send_result {
                        request.sent = true;
                        request.sent_time = Some(Instant::now());
                    } else {
                        info!("Failed to send download request to {:?}", request.from);
                    }
                }
            }

            // Process incoming messages
            _ = process_interval.tick() => {
                let (download_dir, submitted_download_requests) = {
                    let app_opt = app.lock().await;
                    (app_opt.download_dir.clone(), app_opt.requested_files.iter().filter(|r| r.sent).cloned().collect::<Vec<_>>())
                };

                let socket_opt = DOWNLOAD_SOCKET.lock().await;
                let Some(p_socket) = &*socket_opt else { continue; };
                if submitted_download_requests.is_empty() { continue; }

                let mut socket = p_socket.lock().await;
                let received_messages: Vec<_> = {
                    let mut guard = socket.recv.lock().await;
                    guard.drain(..).collect()
                };

                for message in received_messages {
                    let mut stream = DataStream::default();
                    stream.write(&message.data);

                    // Extract command
                    let command = match stream.stream_out::<String>() {
                        Ok(c) => c,
                        Err(_) => {
                            warn!("Missing command");
                            continue;
                        }
                    };

                    match command.as_str() {
                        // ACK_FILE_REQUEST
                        COMMANDS::ACK_FILE_REQUEST => {
                            let request_id = match stream.stream_out::<String>() {
                                Ok(id) => id,
                                Err(_) => {
                                    info!("Missing request_id for ACK");
                                    continue;
                                }
                            };
                            info!("Received ACK for request '{}'", request_id);
                            let mut app_opt = app.lock().await;
                            if let Some(request) = app_opt.requested_files.iter_mut().find(|r| r.request_id == request_id) {
                                request.accepted = true;
                                request.ack_time = Some(Instant::now());
                                let filename = request.filename.clone();
                                drop(request);
                                app_opt.set_message(format!("Request for '{}' accepted", filename));
                            }
                        }

                        // GETFILE
                        COMMANDS::GETFILE => {
                            let request_id = match stream.stream_out::<String>() {
                                Ok(id) => id,
                                Err(_) => {
                                    info!("Missing request_id for GETFILE");
                                    continue;
                                }
                            };
                            let file_bytes = match stream.stream_out::<Vec<u8>>() {
                                Ok(b) => b,
                                Err(_) => {
                                    info!("Missing file bytes");
                                    continue;
                                }
                            };


                            let mut app_opt = app.lock().await;
                            if let Some(request) = app_opt.requested_files.iter_mut().find(|r| r.request_id == request_id) {
                                let filename = request.filename.clone();
                                let download_path = format!("{}/{}", download_dir.display(), filename);
                                match tokio::fs::write(&download_path, &file_bytes).await {
                                    Ok(_) => info!("Saved '{}' to '{}'", filename, download_path),
                                    Err(e) => debug!("Failed to save '{}': {:?}", filename, e),
                                }
                                request.completed = true;
                                app_opt.set_message(format!("Downloaded file '{}'", filename));
                            }
                        }

                        // Unknown command
                        _ => {
                            warn!("Received unknown command '{}'", command);
                        }
                    }
                }
            }
        }
    }
}