# End-to-end test: boots a NixOS VM with the nixos-module services, records an
# event through events-api, reads it back, and fetches a badge from badges.
# Build it with `nix build .#integration-test` (Linux with KVM).
{
  lib,
  testers,
  nixosModule,
}:

let
  test = testers.runNixOSTest {
    name = "commit-integration";

    nodes.machine =
      { pkgs, ... }:
      {
        imports = [ nixosModule ];
        services.commit.events-api.enable = true;
        services.commit.badges.enable = true;
        environment.systemPackages = [ pkgs.curl ];
      };

    testScript = ''
      machine.wait_for_unit("commit-events-api.service")
      machine.wait_for_unit("commit-badges.service")
      machine.wait_for_open_port(3000)
      machine.wait_for_open_port(8081)

      with subtest("events-api stores and filters events"):
          machine.succeed(
              "curl -sf -X POST -H 'content-type: application/json' "
              "-d '{\"repo\":\"o/r\",\"kind\":\"push\",\"actor\":\"ada\"}' "
              "http://127.0.0.1:3000/events"
          )
          events = machine.succeed("curl -sf 'http://127.0.0.1:3000/events?repo=o/r'")
          assert '"actor":"ada"' in events, events
          machine.fail(
              "curl -sf -X POST -H 'content-type: application/json' "
              "-d '{\"repo\":\"bad\",\"kind\":\"push\",\"actor\":\"ada\"}' "
              "http://127.0.0.1:3000/events"
          )

      with subtest("events survive a restart (SQLite in the state directory)"):
          machine.systemctl("restart commit-events-api.service")
          machine.wait_for_open_port(3000)
          events = machine.succeed("curl -sf 'http://127.0.0.1:3000/events?repo=o/r'")
          assert '"actor":"ada"' in events, events

      with subtest("badges renders SVG"):
          svg = machine.succeed("curl -sf http://127.0.0.1:8081/badge/ci/passing.svg")
          assert svg.startswith("<svg") and "passing" in svg, svg
    '';
  };
in
test
// {
  meta = (test.meta or { }) // {
    description = "NixOS VM test of events-api and badges via the nixos-module";
    platforms = lib.platforms.linux;
  };
}
