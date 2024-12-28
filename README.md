# nix-bundle-darwin

A darwin-compatible alternative to [nix-bundle](https://github.com/nix-community/nix-bundle).

## Usage

```
Usage: nix-bundle-darwin [OPTIONS] [TARGET]

Arguments:
  [TARGET]  What to bundle, interpretation depends on mode. Default: must be a path, defaults to "default.nix"; --flake: must be a flake installable, defaults to ".#default"; --program: must be a program name, no default value

Options:
  -A, --attr <ATTR>  Which attribute path of TARGET to build
  -p, --program      Treat TARGET as program, e.g., teeworlds
  -F, --flake        Treat TARGET as flake installable, e.g., nixpkgs#teeworlds
  -f, --force        Overwrite existing bundles
  -h, --help         Print help
  -V, --version      Print version
```

Example: `cargo run --release -- --flake nixpkgs#zed-editor`

## Confirmed working

- zed-editor (Rust)
- teeworlds (C, SDL)
- gg-jj (Tauri)

## Not yet working

- Any Electron app (vscodium, vesktop, ...)
- Anything relying on Nix store paths besides symlinks and shared libraries

## License

EUPL-1.2-or-later

## Differences to [nix-bundle-macos](https://github.com/ariutta/nix-bundle-macos)

- Written in Rust instead of shell scripts
- Does not require `sudo`
- Compatible with flakes
- Shallow copies dependencies from Nix store for smaller bundle size – nix-bundle-macos copies whole store directories based on a nix-store query, resulting in huge bundles
- Unrestricted app location – nix-bundle-macos requires apps to be in `/Applications/`
- Planned: codesign support
