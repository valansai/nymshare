# NymShare

NymShare is a **peer-to-peer file-sharing application** built with Rust and the Nym network. It provides a graphical user interface (GUI) allowing users to **share and download files** over the Nym mixnet. The application supports **drag-and-drop file selection**, **customizable download directories**, and **real-time request tracking**.

## Features
- Anonymous peer-to-peer file sharing over the Nym mixnet.
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

### Share Files
1. Navigate to the **Share** tab.  
2. Add files via drag-and-drop or the **Add Files** button.  
3. Activate files for sharing and copy the generated NymShare link:
4. Share the link with others

![Alt text](https://forum.nym.com/uploads/default/original/2X/c/cc594efe2d2f70af752f117872102e59f1f3acbd.png)



### Download Files
1. Go to the **Download** tab.  
2. Paste a NymShare link (format: `<service_addr>::<filename>`) and click **Download**.  
3. Monitor download progress in the **Download Requests** tab.

![Alt text](https://forum.nym.com/uploads/default/original/2X/9/9bb9fd813b5f7397b904989fa7a7a9f1ecb24849.png)


![Alt text](https://forum.nym.com/uploads/default/original/2X/a/a44310bb1736e76b5f17ba5b82211edc85c2b509.png)



### Explore Files
1. Go to the **Explore** tab.  
2. Paste a Nym address link and click **Explore**.  
3. Monitor explore progress.
4. Search for file names and download them

![Alt text](https://forum.nym.com/uploads/default/original/2X/7/74a9cb17cb4b77e7ee13e3e67c2da2952ad8b0f5.png)

To avoid displaying every advertised file for each explore request, only the count of files is shown. With this search feature, you can search for a file name and check if it appears in one or more of the advertised files. (Feature show all advertise files comming on next release)

![Alt text](https://forum.nym.com/uploads/default/original/2X/6/6598cea14db163d4b19f065463bfc45db748dd60.png)


### Customize Settings
- Change the download directory in the **Download** tab settings.  
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

- **Download Socket (Anonymous Mode)**  
  The client’s Nym address is **never exposed** to the server.  
  - Uses **Single-Use Reply Blocks (SURBs)** to request and receive files while preserving privacy.  
  - SURBs enable servers to respond without ever knowing the client’s Nym address.  
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
