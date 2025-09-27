inputs@{
  config,
  pkgs,
  lib,
  ...
}:

let
  package = import ./package.nix inputs;
  config-format = pkgs.formats.toml { };
  package-wrapper = instance-name: pkgs.writeShellScriptBin "tg-public-log-parser-wrapper" ''
    cd /etc/tg-public-log-parser.d/${instance-name}
    exec ${package}/bin/tg-public-log-parser
  '';
in
{
  options.services.tg-public-log-parser = lib.mkOption {
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
    );
  };

  config = {
    environment.etc = lib.mapAttrs' (instance-name: instance-config: 
      lib.mkIf instance-config.enable
      {
        name = "tg-public-log-parser.d/${instance-name}/config.toml";
        value = {
          source = config-format.generate "config" instance-config.config;
          mode = "0444";
        };
      }) config.services.tg-public-log-parser;

    systemd.services = lib.mapAttrs' (instance-name: instance-config:
      lib.mkIf instance-config.enable 
      {
        name = "tg-public-log-parser-${instance-name}";
        value = {
          description = "tg-public-log-parser-${instance-name}";
          serviceConfig = {
            Type = "simple";
            DynamicUser = true;
            SupplementaryGroups = instance-config.supplementary-groups;
            ExecStart = "${(package-wrapper instance-name)}/bin/tg-public-log-parser-wrapper";
            KillMode = "control-group";
            KillSignal = "KILL";
            Environment = "RUST_LOG=info";
          };
          wantedBy = [ "multi-user.target" ];
          after = ["network.target"];
        };
      }) config.services.tg-public-log-parser;
  };
}
