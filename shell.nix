{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  packages = with pkgs; [
    cargo
    rustc
  ];

  INSTALL_NAME_TOOL =
    if pkgs.stdenv.hostPlatform.isDarwin then
      "${pkgs.cctools}/bin/install_name_tool"
    else
      "${pkgs.libllvm}/bin/llvm-install-name-tool";
}
