# integration-test

End-to-end test of the platform services: boots a NixOS virtual machine with
[`nixos-module`](../nixos-module), records an event through `events-api`, checks it is
filtered by repository, restarts the service to prove the SQLite data persists, and
fetches an SVG badge from `badges`.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **operate**).
Built with the NixOS test framework; needs Linux with KVM.

## Use

```bash
nix build .#integration-test -L   # runs the VM test (about a minute)
```

CI: [`.github/workflows/pkg-integration-test.yml`](../../.github/workflows/pkg-integration-test.yml), which enables KVM on the runner and runs the test when this folder, the module or the flake changes.
