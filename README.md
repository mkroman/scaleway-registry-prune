<h1 align="center">scaleway-registry-prune</h1>
<div align="center">
  <strong>
    Command-line tool to clean up old images on your Scaleway Container Registry
  </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/scaleway-registry-prune">
    <img src="https://img.shields.io/crates/v/scaleway-registry-prune.svg?style=flat-square" alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/scaleway-registry-prune">
    <img src="https://img.shields.io/crates/d/scaleway-registry-prune.svg?style=flat-square" alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/scaleway-registry-prune">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" />
  </a>
</div>

## Usage

```bash
# Deletes all images older than 3 days
% SCW_TOKEN=abcdef scaleway-registry-prune <namespace>/<image> --keep-within '3 days'

# Deletes all images except for 5 most recent
% SCW_TOKEN=abcdef scaleway-registry-prune <namespace>/<image> --keep-last 5
```
