{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
  
  buildInputs = with pkgs; [
    systemd
    libinput
  ];
  
  # This helps pkg-config find the systemd.pc file
  PKG_CONFIG_PATH = "${pkgs.systemd}/lib/pkgconfig";
}
