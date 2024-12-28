{
  lib,
  rustPlatform,
  stdenv,
  cctools,
  libllvm,
}:
let
  fs = lib.fileset;
  cargoMeta = lib.importTOML ./Cargo.toml;
  cargoPackage = cargoMeta.package;
in
rustPlatform.buildRustPackage {
  pname = cargoPackage.name;
  version = cargoPackage.version;
  src = fs.toSource {
    root = ./.;
    fileset = fs.intersection (fs.gitTracked ./.) (
      fs.unions [
        ./src
        ./Cargo.lock
        ./Cargo.toml
      ]
    );
  };
  cargoLock.lockFile = ./Cargo.lock;
  env.INSTALL_NAME_TOOL =
    if stdenv.hostPlatform.isDarwin then
      "${cctools}/bin/install_name_tool"
    else
      "${libllvm}/bin/llvm-install-name-tool";

  meta = {
    description = "A darwin-compatible alternative to nix-bundle";
    homepage = "https://codeberg.org/niklaskorz/nix-bundle-darwin";
    license = lib.licenses.eupl12;
    maintainers = with lib.maintainers; [
      niklaskorz
    ];
    mainProgram = "nix-bundle-darwin";
    platforms = lib.platforms.darwin ++ lib.platforms.linux;
  };
}
