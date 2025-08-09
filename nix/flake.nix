{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/1afe1f27d35031f9e3366963181711e19a9e85df";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        dioxus_cli = f: p: {
          my-dioxus = p.callPackage ./dioxus_cli.nix {};
        };
        overlays = [(import rust-overlay) dioxus_cli];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        #         harfbuzzNew = pkgs.harfbuzz.overrideAttrs (finalAttrs: previousAttrs: {
        #           version = "10.1.0";
        #           src = pkgs.fetchurl {
        #             url = "https://github.com/harfbuzz/harfbuzz/releases/download/${finalAttrs.version}/harfbuzz-${finalAttrs.version}.tar.xz";
        #             hash = "sha256-bONSDy0ImjPO8PxIMhM0uOC3IUH2p2Nxmqrs0neey4I=";
        #           };
        #         });
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        build_dependencies = with pkgs; [
          pkg-config
          gobject-introspection.dev
          nodejs
        ];
        dependencies = with pkgs; [
          my-dioxus
          at-spi2-atk.dev
          atkmm
          cairo.dev
          gdk-pixbuf.dev
          glib.dev
          gtk3.dev
          harfbuzz.dev
          librsvg
          libsoup_3.dev
          pango.dev
          webkitgtk_4_1.dev
          openssl
          xdotool
          systemd.dev
          imagemagick
        ];
      in
        with pkgs; {
          devShells.default = mkShell {
            nativeBuildInputs = [build_dependencies];
            buildInputs = [rustToolchain dependencies];
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        }
    );
}
