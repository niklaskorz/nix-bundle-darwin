# nix-bundle-darwin

A darwin-compatible alternative to [nix-bundle](https://github.com/nix-community/nix-bundle).

## Usage

```
cargo run --release -- --flake <installable> [--force]
```

Example: `cargo run --release -- --flake nixpkgs#zed-editor`

## Confirmed working

- zed-editor (Rust)
- teeworlds (C, SDL)
- gg-jj (Tauri)

## Not yet working

- Any Electron app (vscodium, vesktop, ...)
- Anything relying on Nix store paths 

## License

EUPL-1.2-or-later

## Differences to [nix-bundle-macos](https://github.com/ariutta/nix-bundle-macos)

- Written in Rust instead of shell scripts
- Does not require `sudo`
- Compatible with flakes
- Unrestricted app location (nix-bundle-macos requires apps to be in `/Applications/`)
- Planned: codesign support
