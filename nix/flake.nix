{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
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
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        build_dependencies = with pkgs; [
          pkg-config
          nodejs
        ];
        dependencies = with pkgs; [
          at-spi2-atk.dev
          atkmm
          cairo.dev
          gdk-pixbuf.dev
          glib.dev
          gobject-introspection.dev
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
          dioxus-cli
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
