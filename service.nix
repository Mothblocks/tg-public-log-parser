inputs@{
  pkgs,
  lib,
  ...
}:

let
  package = import ./package.nix inputs;
  service-instances = lib.attrNames config.services.tg-public-log-parser;
  config-format = pkgs.formats.toml { };
  cfg = config.services.tg-public-log-parser;
  package-wrapper = instance-name: pkgs.writeShellScriptBin "tg-public-log-parser-wrapper" ''
    cd /etc/tg-public-log-parser.d/${instance-name}
    exec ${package}/bin/tg-public-log-parser
  '';

  built-config = lib.genAttrs (lib.attrNames cfg) (instance-name: lib.mkIf cfg."${instance-name}".enable {
    environment.etc = {
      "tg-public-log-parser.d/${instance-name}/config.toml" = {
        source = pkgs.formats.toml.generate "config" cfg."${instance-name}";
        mode = "0444";
      };
    };

    systemd.services."tg-public-log-parser-${instance-name}" = {
      description = "tg-public-log-parser-${instance-name}";
      serviceConfig = {
        Type = "simple";
        DynamicUser = true;
        SupplementaryGroups = cfg."${instance-name}".supplementary-groups;
        ExecStart = "${package-wrapper}/bin/tg-public-log-parser-wrapper";
        KillMode = "control-group";
        KillSignal = "KILL";
        Environment = "RUST_LOG=info";
      };
      wantedBy = [ "multi-user.target" ];
      after = ["network.target"];
    };
  });
in
{
  options.services.tg-public-log-parser = {
    enable = lib.mkEnableOption "all tg-public-log-parser instances";
    instances = lib.mkOption {
      description = ''
        Configure instances of the tg-public-log-parser.
      '';
      type = lib.types.attrsOf (
        lib.types.submodule (
          { instance-name, ... }:
          {
            options = {
              enable = lib.mkEnableOption "tg-public-log-parser for ${instance-name}";
              supplementary-groups = lib.mkOption {
                type = lib.types.str;
                default = "";
                description = ''
                  Extra groups to give the service access to.
                '';
              };
              config = lib.mkOption {
                inherit (config-format) type;
                default = { };
                description = lib.mdDoc ''
                  Configuration included in `config.toml`.
                '';
              };
            };
          }
        )
      )
    };
  };

  config = lib.mkIf cfg.enable built-config;
}
