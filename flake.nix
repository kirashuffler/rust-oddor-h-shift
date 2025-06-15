{
  description = "A Nix-flake for a Rust project with systemd service";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, ... }:
  let
  system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
    };
    oddor-h-shift = pkgs.callPackage ./default.nix { };
  in
  {
    packages.${system} = {
      oddor-h-shift = oddor-h-shift;
      default = oddor-h-shift;
    };
    nixosModules.default = {config, lib, pkgs, ...}:{
      imports = [ ./module.nix ];
      config._module.args.oddor-h-shift-package = oddor-h-shift;
    };
  };
}
