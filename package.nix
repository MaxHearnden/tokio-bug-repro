{ lib, mdbook-linkcheck, openssl, pkg-config, rust, rustPlatform, stdenv }:

rustPlatform.buildRustPackage {
  pname = "tokio-dump";
  version = "1.21.2";

  src = ./.;

  # cargoHash = "sha256-Ntrb0jkhz8zl39l7DRQe54WsaDfRK5yBGetS5ltVtJk=";
  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  buildPhase = ''
    cargoBuildHook
    mkdir -p $out
    for output in mir llvm-ir asm; do
      (
      set -x
      cargo rustc --frozen --bin tokio-minimal -p tokio-minimal -F rt-multi-thread --target \
        ${rust.toRustTargetSpec stdenv.hostPlatform} --release -- --emit \
        $output=$out/tokio.$output
      )
    done
    cargoInstallPostBuildHook
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp $bins $out/bin
  '';

  dontCargoInstall = true;

  buildInputs = [
    openssl
  ];

  nativeBuildInputs = [
    pkg-config
  ];

  OPENSSL_NO_VENDOR = 1;

  doCheck = false;

  buildFeatures = [ "rt-multi-thread" ];

  cargoBuildFlags = "-p tokio-minimal";

  meta = {
    mainProgram = "tokio-minimal";
  };
}
