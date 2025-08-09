{
  lib,
  stdenv,
  fetchCrate,
  rustPlatform,
  pkg-config,
  rustfmt,
  cacert,
  openssl,
  nix-update-script,
  testers,
  dioxus-cli,
}:
rustPlatform.buildRustPackage rec {
  pname = "dioxus-cli";
  version = "0.6.3";

  src = fetchCrate {
    inherit pname version;
    hash = "sha256-wuIJq+UN1q5qYW4TXivq93C9kZiPHwBW5Ty2Vpik2oY=";
  };

  cargoHash = "sha256-r4O7DPNqPVxfhbLV8mFJoEYoPW8qb+KLlsJuSASYeVQ=";
  buildFeatures = ["optimizations"];

  nativeBuildInputs = [
    pkg-config
    cacert
  ];

  buildInputs = [openssl];

  OPENSSL_NO_VENDOR = 1;

  nativeCheckInputs = [rustfmt];

  checkFlags = [
    # requires network access
    "--skip=serve::proxy::test"
    # requires fs access
    "--skip=wasm_bindgen::test::test_github_install"
    "--skip=wasm_bindgen::test::test_cargo_install"
  ];

  passthru = {
    updateScript = nix-update-script {};
    tests.version = testers.testVersion {package = dioxus-cli;};
  };

  meta = with lib; {
    homepage = "https://dioxuslabs.com";
    description = "CLI tool for developing, testing, and publishing Dioxus apps";
    changelog = "https://github.com/DioxusLabs/dioxus/releases";
    license = with licenses; [
      mit
      asl20
    ];
    maintainers = with maintainers; [
      xanderio
      cathalmullan
    ];
    mainProgram = "dx";
  };
}
