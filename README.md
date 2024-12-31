# nix-bundle-darwin

A darwin-compatible alternative to [nix-bundle](https://github.com/nix-community/nix-bundle).

## Usage

```
Usage: nix-bundle-darwin [OPTIONS] [INSTALLABLES]... [-- <BUILD_ARGS>...]

Arguments:
  [INSTALLABLES]...  What to bundle. Installables that resolve to derivations are built (or substituted if possible). Store path installables are substituted
  [BUILD_ARGS]...    Additional arguments to pass to `nix build`

Options:
  -f, --file <FILE>  Interpret installables as attribute paths of the Nix expression stored in <FILE>
  -p, --programs     Interpret installables as nixpkgs programs, equivalent to `--file <nixpkgs>`
      --force        Overwrite existing bundles
  -s, --sign         Selfsign the resulting application bundles
  -h, --help         Print help
  -V, --version      Print version
```

Example: `nix-bundle-darwin nixpkgs#zed-editor`

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
- Supports codesigning (currently only selfsign)
