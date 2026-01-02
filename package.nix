{
  pkgs,
  rustPlatform,
  lib,
  ...
}:
rustPlatform.buildRustPackage rec {
  pname = "pull-all";
  version = "0.1.0";
  src = builtins.path {
    name = pname;
    path = ./.;
  };
  cargoHash = "sha256-TsqNl9eDsXZWb/zBAbNgYBROyUUFrrZU/mUEEJ2NW5k=";
}
