{
  lib,
  pkgs,
  tuigreet-pkg,
}: let
  callPackage = lib.callPackageWith (pkgs // {inherit tuigreet-pkg;});
in {
  basic = callPackage ./vm/basic.nix {};
}
