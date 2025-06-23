{
  lib,
  rustPlatform,
  versionCheckHook,
}:

let
  mainProgram = "somo";
in
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "somo";
  version = with builtins; (fromTOML (readFile ../Cargo.toml)).package.version;

  src = lib.fileset.toSource {
    root = ./..;
    fileset = lib.fileset.unions [
      ../src
      ../Cargo.toml
      ../Cargo.lock
    ];
  };

  cargoLock.lockFile = ../Cargo.lock;

  nativeInstallCheckInputs = [
    versionCheckHook
  ];
  doInstallCheck = true;
  versionCheckProgram = "${placeholder "out"}/bin/${mainProgram}";
  versionCheckProgramArg = "--version";

  meta = {
    inherit mainProgram;
    description = "Human-friendly alternative to netstat for socket and port monitoring";
    homepage = "https://github.com/theopfr/somo";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
  };
})
