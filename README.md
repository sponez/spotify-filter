# Spotify Filter

Desktop app for quick Spotify track filtering via global hotkeys.

## Requirements

- Windows 10/11
- Rust toolchain (stable) and Cargo
- Spotify Premium account (required for playback control endpoints)
- A Spotify Developer application

## 1. Create a Spotify App

1. Open Spotify Developer Dashboard and create an app.
2. In app settings, add redirect URI:
   - `http://127.0.0.1:8888/callback`
3. Copy `Client ID` and `Client Secret`.

## 2. Configure environment

Create `.env` in the project root:

```env
SPOTIFY_CLIENT_ID=your_client_id
SPOTIFY_CLIENT_SECRET=your_client_secret
```

## 3. Check configuration

Default config is in `configuration.toml`.

Important sections:
- `app.spotify.api.url`
- `app.spotify.api.paths`
- `app.spotify.auth.redirect_uri`
- `app.spotify.auth.scopes`
- `hotkeys.filter`
- `hotkeys.pass`

Default hotkeys:
- Filter: `Ctrl+Alt+D`
- Pass: `Ctrl+Alt+L`

## 4. Run from source

```powershell
cargo run -p application
```

On first launch, sign in through Spotify in your browser.

## 5. Build release

```powershell
cargo build -p application --release
```

Binary:
- `target/release/application.exe`

To run the built app, keep these files next to the executable:
- `configuration.toml`
- `.env`

The app creates `settings.json` near the executable after first settings save.

## Logging

Default logs are enabled in debug mode.

Custom log level example:

```powershell
$env:RUST_LOG="info,application=debug,core=debug,infrastructure=debug,gui=debug"
cargo run -p application
```

## Notes

- Refresh token is stored in Windows Credential Manager (keyring backend).
- Toast notifications are used for errors and queue sync status.
