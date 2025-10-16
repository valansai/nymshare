# NymShare

NymShare is a **peer-to-peer file-sharing application** built with Rust and the Nym network. It provides a graphical user interface (GUI) allowing users to **share and download files** over the Nym mixnet. The application supports **drag-and-drop file selection**, **customizable download directories**, and **real-time request tracking**.


 ![alt text](https://i.ibb.co/XZhD74Fx/nym-share-front.png)

## Features
- Anonymous peer-to-peer file sharing over the Nym mixnet.
- Switch between Anonymous and Individual download modes
- Easy-to-use GUI built with eframe (egui).
- Drag-and-drop file selection for sharing.
- Customizable download directories.
- Real-time monitoring of download requests.
- Light and dark theme support.


### Build from Source
Clone the repository:
```bash
git clone https://github.com/valansai/nymshare.git
cd nymshare
cargo build --release
```   
   

## Usage

### Launch NymShare
``` bash 
cargo run --release
```

### Sharing Files

1. Go to the **Share** tab.  
2. Add files by dragging and dropping them, or click **Add Files** to upload manually.
3. If you have multiple files added, use the search bar to look up a file by name or hash to locate the file.

   ![Add Files Screenshot](https://i.ibb.co/yBPKmpQN/nym-1.png)
   
4. Right-click any file to activate/deactivate sharing, remove it, or copy its share link.
     
   ![Share Link Screenshot](https://i.ibb.co/jkKtBC4Q/nym-2.png)




### Download Files
1. Go to the **Download** tab.  
2. Paste a NymShare link (format: `<service_addr>::<filename>`) and click **Download**.
3. Watch the complete download files
   
   ![Share Link Screenshot](https://i.ibb.co/wmVWjyR/nym-3.png)

 

4. Monitor download progress in the **Download Requests** tab.
   
   ![Share Link Screenshot](https://i.ibb.co/5XGYbYJW/nym-10.png)

5. If you have many downloaded files, use the search feature to find a specific file by its name or hash.
   
   ![Share Link Screenshot](https://iili.io/KkbgVAN.png)




### Explore Files
1. Go to the **Explore** tab.  
2. Paste a Nym address link and click **Explore**.  
3. Monitor explore progress.
    
   ![Share Link Screenshot](https://i.ibb.co/wNcLGXyX/nym-5.png)

4. Right-click any explore request to show files offered by the service, re-send the request, or remove it.
   
   ![Share Link Screenshot](https://i.ibb.co/tP1XLTn8/nym-6.png)
   
   ![Share Link Screenshot](https://i.ibb.co/wZBMzF89/nym-7.png)

5. On expand you can see the file size and its sha256 hash
   
   ![Share Link Screenshot](https://i.ibb.co/Z1N667Dg/nym-8.png)

6. Right click to download the advertise file
   
   ![Share Link Screenshot](https://i.ibb.co/4RHGmCxc/nym-12.png)

7. Track the download progress on download tab by clickig requests
8. If you have multiple explore requests, each containing many advertised files, use the search bar to look up a file by name or hash to see if any of      the services offer the one you want.

    ![Share Link Screenshot](https://i.ibb.co/gLYHTX5x/nym-13.png)
    
   
   
   
   



### Customize Settings
- Change the download directory in the **Download** tab settings.
- Switch download mode from **Anonymous** to **Individual** and vice-versa in the **Download** tab settings
- Toggle between light and dark themes for the UI.

### Track Requests
- View the status of all download requests (sent, accepted, completed) in the **Download Requests** tab.
- View the status of all explore requests, and search for files in them 

## Key Points
- Only **active files** are available for sharing. Requests for deactivated files will **not be accepted** until the file is reactivated.  
- **Advertisement is optional**; active files can still be served without it.  
- The server keeps track of **download counts** for each file for statistics.  

## Network

- **Serving Socket (Individual Mode)**  
  The server’s Nym address is known to clients. It serves local files by listening for file requests, sending acknowledgments, and transmitting file data.  
  - Serving socket configuration is **stored on disk**, allowing the server to resume operations with the **same Nym address** after a shutdown.

- **Download Socket Default: (Anonymous Mode)**  
  The client’s Nym address is **never exposed** to the server.  
  - Uses **Single-Use Reply Blocks (SURBs)** to request and receive files while preserving privacy.  
  - SURBs enable servers to respond without ever knowing the client’s Nym address.  
  - Download Socket configurations are **ephemeral** and **not stored on disk**, temporary sessions.

- **Download Socket (Individual Mode)**  
  The client’s Nym address is exposed to the server.  
  - Download Socket configurations are **ephemeral** and **not stored on disk**, temporary sessions.


### Background Tasks:
- **serving_manager**: Handles incoming file requests and sends files to requesters.
- **download_manager**: Sends download requests and processes incoming file data, saving files to the specified directory.


## Roadmap
- ✅ **Basic File Sharing**: Drag-and-drop file sharing with links.  
- ✅ **Download Management**: Track download requests and progress.  
- ✅ **Advertise mode**:  
  - Users are able to **explore any NymShare server by address**.  
  - If the server has files available to serve, it eliminates the need to have a specific link.  
  - Servers in **advertisement mode** provide a list of available files, allowing clients to choose which files to download, this makes discovering and downloading content easier, as users no longer need specific file links—knowing the server’s Nym address is enough, In short: Advertise mode replaces the need for per-file links with a discoverable file list on the server.
  - Activate advertise mode on share tab by clicking the settings button.




#
- Nym address: n1cf9fy9wvcp04wdf993qw2fre606ujlxye0yry4
- BTC address: bc1qy6f27lp4aj3jqu3pjmxnaxedhq5uq8g6prg8ru
- XMR address: 46v2JEBdT85Qwna6NZkXZg6wNCQgRTB6VaGRJGGQG8xwBoSzbd4hYCpcZxRFqTcGLZeq3aq64YYkTXJC2eiGWXoaDyhbJRK
