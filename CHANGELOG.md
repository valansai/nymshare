# Changelog

## [0.0.2] - 2025-08-07

### Added
- **Explore Tab**: New tab to discover and download files from remote services.
- **Advertise Mode**: Toggle to control file sharing visibility in the Share tab.
- **Debug Logging**: Added flag for debug logging (partially implemented).
- **Network Commands**: Support for advertising files with new `ADVERTISE` and `GETADVERTISE` commands.
- **Button Styling**: Added macro for consistent button styling across the UI.

### Changed
- Bumped version from 0.0.1 to 0.0.2.
- Reorganized imports for clarity in multiple files.
- Optimized network handling for better performance.
- Renamed `sharable_files` to `shareable_files` for consistency.
- Enhanced UI with settings window in Share tab and improved Explore tab features.

### Fixed
- Improved error handling for network messages.
- Added validation for service addresses in download and explore requests.
- Optimized file filtering and fixed concurrency issues.

### Notes
- Explore tab and advertise mode enhance file discovery and sharing.
- Debug logging will be expanded in future releases.

[0.0.2]: https://github.com/valansai/nymshare/commit/ddebb717bc1cda704a81389bcaebbedf07900c1c
