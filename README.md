## Updating the Flake on /tg/infrastructure

After pushing to main of this repository, run this docker one-liner to generate an updated `flake.lock` file in `<full output dir path>`.

```
docker run -v <full output dir path>:/output --rm nixos/nix bash -c "git clone https://github.com/tgstation-operations/infrastructure && cd infrastructure && nix flake --extra-experimental-features nix-command --extra-experimental-features flakes update tg-public-log-parser && cp flake.lock /output/"
```

PR this updated file to the root of the `main` branch of https://tgstation-operations/infrastructure
