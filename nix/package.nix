{
  lib,
  rustPlatform,
  installShellFiles,
  scdoc,
}: let
  s = ../.;

  cargoTOML = lib.importTOML (s + /Cargo.toml);
in
  rustPlatform.buildRustPackage (finalAttrs: {
    pname = "tuigreet";
    version = cargoTOML.package.version;

    src = let
      fs = lib.fileset;
    in
      fs.toSource {
        root = s;
        fileset = fs.unions [
          (fs.fileFilter (file: builtins.any file.hasExt ["rs"]) (s + /src))
          (s + /contrib)
          (s + /build.rs)
          (s + /Cargo.lock)
          (s + /Cargo.toml)
          (s + /i18n.toml)
        ];
      };

    cargoLock.lockFile = "${finalAttrs.src}/Cargo.lock";
    enableParallelBuilding = true;
    useNextest = true;

    nativeBuildInputs = [
      installShellFiles
      scdoc
    ];

    postInstall = ''
      scdoc < ${../contrib}/man/tuigreet-1.scd > tuigreet.1
      installManPage tuigreet.1
    '';

    meta = {
      description = "Graphical console greeter for greetd";
      license = lib.licenses.gpl3Only;
      maintainers = with lib.maintainers; [NotAShelf];
      mainProgram = "tuigreet";
    };
  })
