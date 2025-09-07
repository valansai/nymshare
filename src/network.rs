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



// External crates
use nymlib::nymsocket::{Socket, SockAddr, SocketMode};
use nymlib::serialize::{DataStream, Serialize};
use tokio::{
    sync::{broadcast, mpsc, Mutex},
    time::{Duration, interval},
};
use log::{debug, info, warn, error};


// Standard library
use std::sync::LazyLock;
use std::sync::Arc;
use std::io::Write;
use std::time::Instant;

// Local 
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
    pub const ADVERTISE: &str = "ADVERTISE";         
    pub const GETADVERTISE: &str = "GETADVERTISE"; 
    pub const ACK_ADVERTISE_REQUEST: &str = "ACK_ADVERTISE_REQUEST";   
        
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

                // Drain messages while holding the lock briefly
                let messages: Vec<_> = {
                    let mut socket_guard = p_socket.lock().await;
                    let mut recv_guard = socket_guard.recv.lock().await;
                    recv_guard.drain(..).collect()
                };

                // Process each message without holding the socket lock
                for message in messages {
                    let mut stream = DataStream::default();
                    stream.write(&message.data);

                    let command = match stream.stream_out::<String>() {
                        Ok(cmd) => cmd,
                        Err(_) => {
                            warn!("Invalid message format: missing command");
                            continue;
                        }
                    };

                    match command.as_str() {
                        COMMANDS::FILE_REQUEST => {
                            info!("[*] Received FILE_REQUEST");

                            let (request_id, requested_file_name) = match (stream.stream_out::<String>(), stream.stream_out::<String>()) {
                                (Ok(id), Ok(name)) => (id, name),
                                (Err(_), _) => { info!("Missing request_id"); continue; },
                                (_, Err(_)) => { info!("Missing filename"); continue; },
                            };

                            let mut app_guard = app.lock().await;
                            let file_opt = app_guard.shareable_files.iter_mut()
                                .find(|f| f.file_name().map(|n| n == requested_file_name).unwrap_or(false) && f.is_active());

                            let Some(file) = file_opt else {
                                info!("File {} not found or inactive", requested_file_name);
                                continue;
                            };

                            let mut socket_guard = p_socket.lock().await;

                            // Send ACK
                            let mut ack_stream = DataStream::default();
                            ack_stream.stream_in(&COMMANDS::ACK_FILE_REQUEST);
                            ack_stream.stream_in(&request_id);
                            if socket_guard.send(ack_stream.data.clone(), message.from.clone()).await {
                                info!("Sent ACK for '{}' (id={})", requested_file_name, request_id);
                            } else {
                                warn!("Failed to send ACK for '{}'", requested_file_name);
                                continue;
                            }

                            // Send file
                            let file_bytes = match file.read_bytes() {
                                Ok(b) => b,
                                Err(e) => { warn!("Failed to read '{}': {:?}", requested_file_name, e); continue; },
                            };

                            let mut out_stream = DataStream::default();
                            out_stream.stream_in(&COMMANDS::GETFILE);
                            out_stream.stream_in(&request_id);
                            out_stream.stream_in(&file_bytes);

                            if socket_guard.send(out_stream.data.clone(), message.from.clone()).await {
                                file.downloads = file.downloads.saturating_add(1);
                                info!("Sent file {} to {:?}", requested_file_name, message.from.to_string());
                            } else {
                                warn!("Failed to send file {}", requested_file_name);
                            }
                        }

                        COMMANDS::ADVERTISE => {
                            info!("[*] Received ADVERTISE");

                            {
                                let mut app_guard = app.lock().await;
                                if !app_guard.advertise_mode {
                                    info!("Skip ADVERTISE, not in advertise mode");
                                    continue;
                                }
                            }

                            let request_id = match stream.stream_out::<String>() {
                                Ok(id) => id,
                                Err(_) => { info!("Missing request_id for ADVERTISE"); continue; },
                            };

                            let mut socket_guard = p_socket.lock().await;

                            // Send ACK
                            let mut ack_stream = DataStream::default();
                            ack_stream.stream_in(&COMMANDS::ACK_ADVERTISE_REQUEST);
                            ack_stream.stream_in(&request_id);
                            if socket_guard.send(ack_stream.data.clone(), message.from.clone()).await {
                                info!("Sent ACK_ADVERTISE_REQUEST for (id={})", request_id);
                            } else {
                                warn!("Failed to send ACK_ADVERTISE_REQUEST for '{}'", request_id);
                                continue;
                            }

                            let mut app_guard = app.lock().await;
                            let shareable_files: Vec<String> = app_guard.shareable_files
                                .iter()
                                .filter(|f| f.is_active())
                                .filter_map(|f| f.file_name().clone())
                                .collect();

                            let mut out_stream = DataStream::default();
                            out_stream.stream_in(&COMMANDS::GETADVERTISE);
                            out_stream.stream_in(&request_id);
                            out_stream.stream_in(&shareable_files);

                            if socket_guard.send(out_stream.data.clone(), message.from.clone()).await {
                                info!("[*] Sent GETADVERTISE {:?} to {:?}", shareable_files, message.from.to_string());
                            } else {
                                info!("[*] Failed to send GETADVERTISE to {:?}", message.from);
                                continue;
                            }

                            // Increment advertise counts
                            for filename in &shareable_files {
                                for f in app_guard.shareable_files.iter_mut() {
                                    if let Some(name) = &f.file_name() {
                                        if name == filename {
                                            f.advertise = f.advertise.saturating_add(1);
                                        }
                                    }
                                }
                            }
                        }

                        _ => {
                            info!("Unknown command received: {}", command);
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
        guard
            .as_ref()
            .ok_or_else(|| "Stop signal not initialized".to_string())?
            .subscribe()
    };

    // Setup intervals
    let mut send_interval = interval(Duration::from_millis(200));
    let mut process_interval = interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            // Stop signal handling
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

            // Send pending download and explore requests
            _ = send_interval.tick() => {
                let socket_opt = DOWNLOAD_SOCKET.lock().await;
                let Some(p_socket) = &*socket_opt else { continue; };

                // Lock socket once for sending all requests
                let mut socket_guard = p_socket.lock().await;

                // Handle download requests
                {
                    let mut app_guard = app.lock().await;
                    for request in app_guard.requested_files.iter_mut().filter(|r| !r.sent) {
                        let mut stream = DataStream::default();
                        stream.stream_in(&COMMANDS::FILE_REQUEST);
                        stream.stream_in(request);
                        let serialized = stream.data.clone();

                        socket_guard.extra_surbs = Some(10);
                        if socket_guard.send(serialized, request.from.clone()).await {
                            request.sent = true;
                            request.sent_time = Some(Instant::now());
                            info!("[*] Sent download request for {:?} to {:?}",
                                request.filename, request.from.to_string());
                        } else {
                            info!("[*] Failed to send download request for {:?} to {:?}",
                                request.filename, request.from.to_string());
                        }
                    }
                }

                // Handle explore requests
                {
                    let mut app_guard = app.lock().await;
                    for request in app_guard.explore_requests.iter_mut().filter(|r| !r.sent) {
                        let mut stream = DataStream::default();
                        stream.stream_in(&COMMANDS::ADVERTISE);
                        stream.stream_in(request);
                        let serialized = stream.data.clone();

                        socket_guard.extra_surbs = Some(5);
                        if socket_guard.send(serialized, request.from.clone()).await {
                            request.sent = true;
                            request.sent_time = Some(Instant::now());
                            info!("[*] Sent explore request to {:?}", request.from.to_string());
                        } else {
                            info!("[*] Failed to send explore request to {:?}", request.from.to_string());
                        }
                    }
                }
            }

            // Process incoming messages
            _ = process_interval.tick() => {
                let socket_opt = DOWNLOAD_SOCKET.lock().await;
                let Some(p_socket) = &*socket_opt else { continue; };

                // Lock socket only while draining messages
                let messages: Vec<_> = {
                    let mut socket_guard = p_socket.lock().await;
                    let mut recv_guard = socket_guard.recv.lock().await;
                    recv_guard.drain(..).collect()
                };

                for message in messages {
                    let mut stream = DataStream::default();
                    stream.write(&message.data);

                    // Extract command
                    let command = match stream.stream_out::<String>() {
                        Ok(c) => c,
                        Err(_) => {
                            warn!("Invalid message format: missing command");
                            continue;
                        }
                    };

                    match command.as_str() {
                        COMMANDS::ACK_FILE_REQUEST => {
                            let request_id = match stream.stream_out::<String>() {
                                Ok(id) => id,
                                Err(_) => { info!("Missing request_id for ACK"); continue; }
                            };
                            info!("Received ACK for request '{}'", request_id);

                            let mut app_guard = app.lock().await;
                            if let Some(req) = app_guard.requested_files.iter_mut()
                                .find(|r| r.request_id == request_id) {
                                req.accepted = true;
                                req.ack_time = Some(Instant::now());
                                let filename = req.filename.clone();
                                drop(req);
                                app_guard.set_message(format!("Request for '{}' accepted", filename));
                            }
                        }

                        COMMANDS::ACK_ADVERTISE_REQUEST => {
                            let request_id = match stream.stream_out::<String>() {
                                Ok(id) => id,
                                Err(_) => { 
                                    info!("Missing request_id for ACK"); 
                                    continue; 
                                }
                            };
                            info!("Received ACK_ADVERTISE_REQUEST for request '{}'", request_id);

                            let mut app_guard = app.lock().await;
                            if let Some(req) = app_guard.explore_requests.iter_mut()
                                .find(|r| r.request_id == request_id) 
                            {
                                if !req.accepted {
                                    req.accepted = true;
                                    req.ack_time = Some(Instant::now());
                                    app_guard.set_message(format!(
                                        "ACK_ADVERTISE_REQUEST for '{}' accepted", request_id
                                    ));
                                } else {
                                    info!(
                                        "ACK_ADVERTISE_REQUEST for '{}' arrived late (already accepted earlier)",
                                        request_id
                                    );
                                }
                            }
                        }

                        COMMANDS::GETFILE => {
                            let request_id = match stream.stream_out::<String>() {
                                Ok(id) => id,
                                Err(_) => { info!("Missing request_id for GETFILE"); continue; }
                            };
                            let file_bytes = match stream.stream_out::<Vec<u8>>() {
                                Ok(b) => b,
                                Err(_) => { info!("Missing file bytes"); continue; }
                            };

                            let download_dir = app.lock().await.download_dir.clone();

                            let mut app_guard = app.lock().await; 
                            if let Some(req) = app_guard.requested_files.iter_mut()
                                .find(|r| r.request_id == request_id) {
                                
                                let filename = req.filename.clone(); 
                                let download_path = format!("{}/{}", download_dir.display(), filename);

                                match tokio::fs::write(&download_path, &file_bytes).await {
                                    Ok(_) => info!("Saved '{}' to '{}'", filename, download_path),
                                    Err(e) => debug!("Failed to save '{}': {:?}", filename, e),
                                }

                                req.completed = true;
                                app_guard.set_message(format!("Downloaded file '{}'", filename));
                            }
                        }

                        COMMANDS::GETADVERTISE => {
                            let request_id = match stream.stream_out::<String>() {
                                Ok(id) => id,
                                Err(_) => { info!("Missing request_id for GETADVERTISE"); continue; }
                            };
                            let file_names = match stream.stream_out::<Vec<String>>() {
                                Ok(names) => names,
                                Err(_) => { info!("Missing file names for GETADVERTISE"); continue; }
                            };
                            info!("[*] Received GETADVERTISE for request '{}': {:?}", request_id, file_names);


                            let mut app_guard = app.lock().await;
                            if let Some(req) = app_guard.explore_requests.iter_mut()
                                    .find(|r| r.request_id == request_id) 
                                {
                                    if !req.accepted {
                                        req.accepted = true;
                                        req.ack_time = Some(Instant::now());
                                        info!("No ACK received before GETADVERTISE; auto-marking ACK at {:?}", req.ack_time);
                                    }

                                    req.advertise_files = file_names.clone();
                                    req.completed = true;
                                    app_guard.set_message(format!("Discovered files for '{}'", request_id));
                                }
                            }
                        _ => {
                            warn!("[*] Unknown command received: '{}'", command);
                        }
                    }
                }
            }
        }
    }
}