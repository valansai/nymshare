# NymShare

NymShare is a **peer-to-peer file-sharing application** built with Rust and the Nym network. It provides a graphical user interface (GUI) allowing users to **share and download files** over the Nym mixnet. The application supports **drag-and-drop file selection**, **customizable download directories**, and **real-time request tracking**.


<img src="https://i.ibb.co/XZhD74Fx/nym-share-front.png" alt="Nym share front" height=400 width="900">


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


<img src="https://i.ibb.co/HfK5L9k0/nym-share-1.png" alt="Nym share front" height=400 width="1000">




### Download Files
1. Go to the **Download** tab.  
2. Paste a NymShare link (format: `<service_addr>::<filename>`) and click **Download**.
3. Watch the complete download files 


<img src="https://i.ibb.co/xKHZxH1Y/nym-share-001.png" alt="Nym share front" height=400 width="1000">

3. Monitor download progress in the **Download Requests** tab.
<img src="https://i.ibb.co/6RgLgz3j/nym-share-002.png" alt="Nym share front" height=400 width="1000">




### Explore Files
1. Go to the **Explore** tab.  
2. Paste a Nym address link and click **Explore**.  
3. Monitor explore progress.
4. Search for file names and download them


<img src="https://i.ibb.co/7dTMX82L/nym-share-0001.png" alt="Nym share front" height=400 width="1000">


To avoid listing every advertised file for each explore request, click Show to view which files are offered by a specific service address. If you have multiple explore requests, simply search by file name to check whether it appears in one or more of the advertised requests.

<img src="https://i.ibb.co/CKzJbjCD/nym-share-0002.png" alt="Nym share front" height=400 width="1000">


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
