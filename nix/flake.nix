{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    # nixpkgs-webkit.url = "github:nixos/nixpkgs/19f22b217c2753ba5e6885c7967bad5337b36c1d";
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
    # nixpkgs-webkit,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        # dioxus_cli = f: p: {
        #   my-dioxus = p.callPackage ./dioxus_cli.nix {};
        # };
        # webkitOverlay = final: prev: let
        #   pkgs-webkit = import nixpkgs-webkit {
        #     inherit system;
        #     inherit (prev) config;
        #   };
        # in {
        #   webkitgtk_4_1 = pkgs-webkit.webkitgtk_4_1;
        # };
        # overlays = [(import rust-overlay) dioxus_cli webkitOverlay];
        overlays = [(import rust-overlay)];
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
