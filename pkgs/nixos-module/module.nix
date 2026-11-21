# NixOS module that runs the commit platform services as hardened systemd
# units: services.commit.events-api (SQLite in /var/lib) and services.commit.badges.
# Exposed by the flake as nixosModules.default.
{ self }:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.commit;
  flakePackages = self.packages.${pkgs.stdenv.hostPlatform.system};

  service =
    {
      name,
      description,
      defaultPort,
    }:
    {
      enable = lib.mkEnableOption description;
      package = lib.mkOption {
        type = lib.types.package;
        default = flakePackages.${name};
        defaultText = lib.literalExpression "commit.packages.\${system}.${name}";
        description = "Package providing the ${name} binary.";
      };
      address = lib.mkOption {
        type = lib.types.str;
        default = "127.0.0.1";
        description = "Address ${name} listens on.";
      };
      port = lib.mkOption {
        type = lib.types.port;
        default = defaultPort;
        description = "Port ${name} listens on.";
      };
      openFirewall = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Open the port in the firewall.";
      };
    };

  unit = name: opts: extraEnv: {
    description = "commit ${name}";
    wantedBy = [ "multi-user.target" ];
    after = [ "network.target" ];
    environment = {
      ADDR = "${opts.address}:${toString opts.port}";
    }
    // extraEnv;
    serviceConfig = {
      ExecStart = lib.getExe' opts.package name;
      DynamicUser = true;
      StateDirectory = "commit-${name}";
      Restart = "on-failure";
      NoNewPrivileges = true;
      ProtectSystem = "strict";
      ProtectHome = true;
      PrivateTmp = true;
      RestrictAddressFamilies = [
        "AF_INET"
        "AF_INET6"
      ];
    };
  };
in
{
  options.services.commit = {
    events-api = service {
      name = "events-api";
      description = "the commit events API";
      defaultPort = 3000;
    };
    badges = service {
      name = "badges";
      description = "the commit badge service";
      defaultPort = 8081;
    };
  };

  config = lib.mkMerge [
    (lib.mkIf cfg.events-api.enable {
      systemd.services.commit-events-api = unit "events-api" cfg.events-api {
        DATABASE_URL = "sqlite:///var/lib/commit-events-api/events.db?mode=rwc";
      };
      networking.firewall.allowedTCPPorts = lib.optional cfg.events-api.openFirewall cfg.events-api.port;
    })
    (lib.mkIf cfg.badges.enable {
      systemd.services.commit-badges = unit "badges" cfg.badges { };
      networking.firewall.allowedTCPPorts = lib.optional cfg.badges.openFirewall cfg.badges.port;
    })
  ];
}
