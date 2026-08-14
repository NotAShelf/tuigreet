{
  lib,
  craneLib,
  installShellFiles,
  versionCheckHook,
  scdoc,
}: let
  commonArgs = {
    pname = "tuigreet";
    version = (lib.importTOML ../Cargo.toml).workspace.package.version;
    src = let
      s = ../.;
      fs = lib.fileset;
    in
      fs.toSource {
        root = s;
        fileset = fs.unions [
          (s + /contrib)
          (s + /crates)
          (s + /Cargo.lock)
          (s + /Cargo.toml)
        ];
      };
    strictDeps = true;
  };

  cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {name = "tuigreet-deps";});
in
  craneLib.buildPackage (commonArgs
    // {
      inherit cargoArtifacts;
      useNextest = true;

      nativeInstallCheckInputs = [versionCheckHook];
      doInstallCheck = true;

      nativeBuildInputs = [
        installShellFiles
      ];

      postInstall = ''
        installManPage ${../contrib}/man/tuigreet.1
      '';

      meta = {
        description = "Stylish graphical console greeter for greetd";
        license = lib.licenses.gpl3Only;
        maintainers = with lib.maintainers; [NotAShelf];
        mainProgram = "tuigreet";
      };
    })
