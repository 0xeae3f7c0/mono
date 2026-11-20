# Security Policy

## Reporting a vulnerability

Do not report security vulnerabilities in public issues or pull requests.

Use GitHub's **private vulnerability reporting** (Security tab → "Report a vulnerability")
if it is enabled, or contact the repository owner privately through GitHub. Include the
affected project in `pkgs/` (or root tool), the version or commit, and steps to reproduce.

## Supported versions

Only the latest commit on `main` is supported. Dependencies are kept current by the
Repository Maintenance workflow; security fixes land the same way, one pull request per
dependency version.

## Guidelines for contributors

- Never commit tokens, credentials, private keys or `.env` files.
- Webhook endpoints must verify signatures (see `pkgs/webhooks`); never trust unsigned
  payloads.
- Workflows request only the permissions they need; secrets such as `OWNER_TOKEN` are
  read only by the workflows that use them.
- Automated review comments and CodeQL results are signals, not security certification
  or human approval.
