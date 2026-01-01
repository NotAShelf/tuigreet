{
  description = "Stylish graphical console greeter for greetd, built with Ratatui ";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";

  outputs = {
    self,
    nixpkgs,
  }: let
    inherit (nixpkgs) lib;
    systems = ["x86_64-linux" "aarch64-linux"];
    forEachSystem = lib.genAttrs systems;
    pkgsForEach = nixpkgs.legacyPackages;
  in {
    packages = forEachSystem (system: {
      tuigreet = pkgsForEach.${system}.callPackage ./nix/package.nix {};
      default = self.packages.${system}.tuigreet;
    });

    devShells = forEachSystem (system: {
      default = pkgsForEach.${system}.callPackage ./nix/shell.nix {};
    });

    checks = forEachSystem (system: let
      pkgs = pkgsForEach.${system};
      tests = pkgs.callPackage ./nix/tests/default.nix {inherit (self.packages.${system}) tuigreet;};
    in
      # Expose each test as a separate check, filtering out override functions
      lib.filterAttrs (name: _: !builtins.elem name ["override" "overrideDerivation"]) tests);

    # In the case I ever decide to use Hydra
    hydraJobs = self.packages;
  };
}
