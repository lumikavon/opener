# Opener - Desktop Launcher Application

A powerful, offline-capable desktop launcher application built with Tauri v2, HTML5/CSS/JS, TailwindCSS, and SQLite.

## Features

- **Multiple Entry Types**: Support for App, URL, File, Directory, Command, WSL, SSH, Script, Shortcut (.lnk), and AutoHotkey
- **Fuzzy Search**: Fast, debounced fuzzy search across all entries
- **Keyboard Navigation**: Full keyboard support with arrow keys and Enter to execute
- **Script Templates**: Pre-built and custom templates with variable substitution
- **Hotkey Bindings**: Bind keyboard shortcuts to entries (application-level and global)
- **Import/Export**: Backup and restore configuration with multiple merge strategies
- **Secure Credential Storage**: SSH keys and sensitive data stored in system keychain
- **Fully Offline**: All dependencies bundled locally, no CDN or internet required
- **Cross-Platform**: Windows (primary), macOS, and Linux support

## Screenshots

The application features a clean, modern interface with:
- Frameless window with custom drag region
- Real-time fuzzy search
- Settings modal with tabbed navigation
- Type-specific entry icons and badges

## Requirements

### Development Environment

- **Rust**: 1.70 or later
- **Node.js**: 18.x or later (24.x recommended)
- **npm**: 8.x or later

### Platform-Specific Requirements

#### Windows
- Windows 10/11
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed on Windows 10/11)
- For .lnk parsing: Windows Shell APIs (built-in)
- For AHK support: [AutoHotkey](https://www.autohotkey.com/) installed

#### macOS
- macOS 10.15 (Catalina) or later
- Xcode Command Line Tools

#### Linux
- WebKitGTK and related libraries
- For Debian/Ubuntu:
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libappindicator3-dev librsvg2-dev patchelf
  ```

## Installation

### From Source

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-org/opener.git
   cd opener
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Build TailwindCSS (local build, no CDN)**
   ```bash
   npm run tailwind:build
   ```

4. **Run in development mode**
   ```bash
   npm run tauri:dev
   ```

5. **Build for production**
   ```bash
   npm run tauri:build
   ```

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/your-org/opener/releases) page.

## Building & Packaging

### Development

```bash
# Install dependencies
npm install

# Start development server with hot reload
npm run tauri:dev
```

### Production Build

```bash
# Build frontend and Tauri application
npm run tauri:build
```

### Platform-Specific Builds

#### Windows (MSI/NSIS)
```bash
npm run tauri:build
# Output: src-tauri/target/release/bundle/msi/Opener_1.0.2_x64_en-US.msi
# Output: src-tauri/target/release/bundle/nsis/Opener_1.0.2_x64-setup.exe
```

#### macOS (DMG/App Bundle)
```bash
npm run tauri:build
# Output: src-tauri/target/release/bundle/dmg/Opener_1.0.2_x64.dmg
# Output: src-tauri/target/release/bundle/macos/Opener.app
```

#### Linux (AppImage/Deb)
```bash
npm run tauri:build
# Output: src-tauri/target/release/bundle/appimage/opener_1.0.2_amd64.AppImage
# Output: src-tauri/target/release/bundle/deb/opener_1.0.2_amd64.deb
```

### Build Output Locations

| Platform | Format | Location |
|----------|--------|----------|
| Windows | MSI | `src-tauri/target/release/bundle/msi/` |
| Windows | NSIS | `src-tauri/target/release/bundle/nsis/` |
| macOS | DMG | `src-tauri/target/release/bundle/dmg/` |
| macOS | App | `src-tauri/target/release/bundle/macos/` |
| Linux | AppImage | `src-tauri/target/release/bundle/appimage/` |
| Linux | Deb | `src-tauri/target/release/bundle/deb/` |

## Usage Guide

### Adding Entries

1. Open Settings (gear icon or press the settings button)
2. Navigate to "Entries" tab
3. Click "Add Entry"
4. Select the entry type and fill in the required fields:
   - **App**: Path to executable, optional arguments
   - **URL**: Web address
   - **File**: Path to any file
   - **Dir**: Path to directory
   - **Cmd**: Command to execute
   - **WSL**: Command to run in Windows Subsystem for Linux
   - **SSH**: Host, user, port configuration
   - **Script**: Script content or path
   - **Shortcut**: Path to .lnk file (Windows)
   - **AHK**: AutoHotkey script path or content (Windows)

### Keyboard Shortcuts

- **↑/↓**: Navigate search results
- **Enter**: Execute selected entry
- **Esc**: Clear search / Close modal
- Custom hotkeys can be configured per entry

### Binding Hotkeys

1. Open Settings
2. Navigate to "Hotkeys" tab
3. Click "Add Hotkey"
4. Select an entry from the dropdown
5. Press your desired key combination
6. Choose scope (Application or Global)

### Using Script Templates

1. Open Settings
2. Navigate to "Templates" tab
3. Click the play button on a template to use it
4. Fill in the variable values
5. Click "Create Entry" to save as a new entry

Templates use `{{variable_name}}` syntax for variables.

### Import/Export

1. Open Settings
2. Navigate to "Import/Export" tab
3. **Export**: Click "Export to File" to save configuration
4. **Import**: Select strategy and click "Import from File"

Import strategies:
- **Add only**: Only add new entries, skip existing
- **Merge by name**: Update existing entries with matching names
- **Overwrite all**: Replace all data with imported data

## Configuration

### Display Settings

- **Icon Size**: 16/24/32/48 pixels
- **Show Path**: Toggle target path visibility
- **Show Type Label**: Toggle type badge visibility
- **Show Description**: Toggle description visibility
- **Sort Strategy**: Relevance/Name/Recently Used/Most Used
- **Max Results**: 10/20/50/100 entries

### Security Settings

- **Confirm dangerous commands**: Prompt before executing Cmd/WSL/SSH/Script/AHK entries

## Offline Dependencies

This application is fully offline-capable:

- **TailwindCSS**: Built locally during the build process
- **All JavaScript**: Bundled with the application
- **Icons**: SVG icons embedded in HTML
- **Fonts**: System fonts used
- **No external CDN references**

To verify offline capability:
1. Build the application
2. Disconnect from the internet
3. Run the built application

## Database

The application uses SQLite for data persistence. The database is stored at:

- **Windows**: `%APPDATA%\com.opener.app\opener.db`
- **macOS**: `~/Library/Application Support/com.opener.app/opener.db`
- **Linux**: `~/.local/share/com.opener.app/opener.db`

### Database Schema

```sql
-- Entries table
CREATE TABLE entries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    target TEXT NOT NULL,
    args TEXT,
    workdir TEXT,
    icon_path TEXT,
    tags TEXT,
    description TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    confirm_before_run INTEGER,
    show_terminal INTEGER,
    wsl_distro TEXT,
    ssh_host TEXT,
    ssh_user TEXT,
    ssh_port INTEGER,
    ssh_key_id TEXT,
    env_vars TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_used_at TEXT,
    use_count INTEGER NOT NULL DEFAULT 0
);

-- Hotkeys table
CREATE TABLE hotkeys (
    id TEXT PRIMARY KEY,
    entry_id TEXT NOT NULL,
    accelerator TEXT NOT NULL,
    scope TEXT NOT NULL DEFAULT 'app',
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE
);

-- Settings table
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Script templates table
CREATE TABLE script_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    language TEXT NOT NULL,
    template_content TEXT NOT NULL,
    variables_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

## Running Tests

```bash
# Run Rust unit tests
npm test
# or
cargo test --manifest-path src-tauri/Cargo.toml
```

Tests cover:
- Database CRUD operations
- Search functionality
- Import/Export
- Template rendering
- Hotkey conflict detection

## Security Considerations

### Dangerous Commands

Commands (Cmd, WSL, SSH, Script, AHK) can execute arbitrary code. By default, the application prompts for confirmation before executing these types. This can be:
- Disabled globally in Display settings
- Disabled per-entry in the entry configuration

### Credential Storage

SSH keys and other sensitive credentials are stored using the system keychain:
- **Windows**: Windows Credential Manager
- **macOS**: Keychain
- **Linux**: Secret Service (GNOME Keyring / KDE Wallet)

Credentials are never stored in plain text in the database.

## Platform Limitations

### AutoHotkey (Windows Only)
- Requires AutoHotkey to be installed
- Configure AHK executable path if not in PATH

### .lnk Shortcuts (Windows Only)
- Full support for parsing Windows shortcut files
- On other platforms, the file will be opened with the default handler

### WSL (Windows Only)
- Requires Windows Subsystem for Linux to be installed
- Configure distribution name in entry settings

## Troubleshooting

### Build Errors

1. **Missing WebView2**: Install WebView2 Runtime on Windows
2. **Missing system libraries**: Install required packages for your Linux distribution
3. **Rust version**: Ensure Rust 1.70+ is installed

### Runtime Issues

1. **Database errors**: Delete the database file to reset (see paths above)
2. **Hotkeys not working**: Check for conflicts with other applications
3. **AHK not found**: Install AutoHotkey or configure the path in settings

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `npm test`
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) - Cross-platform desktop framework
- [TailwindCSS](https://tailwindcss.com/) - Utility-first CSS framework
- [SQLite](https://sqlite.org/) - Embedded database engine
