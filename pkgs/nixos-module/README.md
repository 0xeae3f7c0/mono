# nixos-module

NixOS module that runs the commit services as hardened systemd units: `events-api`
(with its SQLite database in `/var/lib/commit-events-api`) and `badges`. A way to
deploy without Kubernetes.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Uses the packages this flake builds; exposed as `nixosModules.default`.

## Use

```nix
{
  inputs.commit.url = "github:0xeae3f7c0/mono";

  outputs = { nixpkgs, commit, ... }: {
    nixosConfigurations.server = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        commit.nixosModules.default
        {
          services.commit.events-api = { enable = true; address = "0.0.0.0"; openFirewall = true; };
          services.commit.badges.enable = true;
        }
      ];
    };
  };
}
```

Options per service: `enable`, `package`, `address`, `port`, `openFirewall`.
[`integration-test`](../integration-test) boots both services in a VM.

CI: [`.github/workflows/pkg-nixos-module.yml`](../../.github/workflows/pkg-nixos-module.yml) evaluates the module.
