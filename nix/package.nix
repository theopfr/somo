{
  lib,
  stdenv,
  rustPlatform,
  installShellFiles,
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

  nativeBuildInputs = [
    installShellFiles
  ];

  postInstall = lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    installShellCompletion --cmd ${mainProgram} \
      --bash <("$out/bin/${mainProgram}" generate-completions bash) \
      --zsh <("$out/bin/${mainProgram}" generate-completions zsh) \
      --fish <("$out/bin/${mainProgram}" generate-completions fish)
  '';

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
