{ pkgs, lib, rustPlatform, stdenv }:

rustPlatform.buildRustPackage rec {
  pname = "oddor-h-shift";
  version = "1.0.0";

  src = lib.cleanSourceWith {
    src = ./.;
    filter = path: type:
      let
        baseName = baseNameOf path;
      in
      !(
        baseName == "flake.nix" ||
        baseName == "flake.lock" ||
        (type == "directory" && baseName == "target")
      );
  };

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = with pkgs; [
    pkg-config
    systemd
  ];
  
  buildInputs = with pkgs; [
    libinput
  ];

  RUST_BACKTRACE = 1;
  PKG_CONFIG_PATH = "${pkgs.systemd}/lib/pkgconfig";
}
