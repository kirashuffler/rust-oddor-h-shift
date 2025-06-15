{ lib, config, pkgs, oddor-h-shift-package, ... }:

with lib;

let
  cfg = config.services.oddor-h-shift;
  package = oddor-h-shift-package;
in
{
  options.services.oddor-h-shift = {
    enable = mkEnableOption "Oddor H Shift driver service";
  };

  config = mkIf cfg.enable {
    systemd.services.oddor-h-shift = {
      description = "Oddor H Shift driver";
      wantedBy = [ "default.target" ];
      # after = [ "network.target" ];

      serviceConfig = {
        ExecStart = "${package}/bin/oddor_h_shift";
        Restart = "always";
      };
    };
  };
}
