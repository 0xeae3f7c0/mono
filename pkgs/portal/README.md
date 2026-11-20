# portal

Developer portal cataloguing every service here.

Part of the [commit](../../ARCHITECTURE.md) platform (stage: **show**).
Built with Backstage.

## Develop

```bash
corepack enable
yarn install
yarn start
yarn ci   # type-check and tests, as CI runs them
```

CI: [`.github/workflows/pkg-portal.yml`](../../.github/workflows/pkg-portal.yml), which runs when files in this folder change.
