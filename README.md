# Arknights assets

A rust-based project for downloading and unpacking assets for Arknights.

This is a rewrite for the complete asset-pipeline as used in
[myrtle.moe](https://github.com/Eltik/myrtle), including

- downloading assets from the server
- unpacking the downloaded files
- extracting the actual assets
- decoding the text assets
- combining the images
- splitting the sprite sheets

---

## Usage

Dependencies

- [Docker engine](https://docs.docker.com/engine/)
- [Docker compose](https://docs.docker.com/compose/)

```shell
# Build the image with the CLI
docker compose build


# Then, the CLI can be called using
docker compose run --rm pipeline --help


# For example, the downloader can be run with
docker compose run --rm pipeline download


# Use the help command to learn more about the CLI.
docker compose run --rm pipeline help download
```
