{
  description = "DevShell and build configuration for moccacino";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain pkgs.rust-bin.nightly.latest.default;

        buildInputs = with pkgs; [
          pkg-config
          openssl
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          xorg.libxcb
          libxkbcommon
          vulkan-loader
          wayland
        ];

        moccacino = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          inherit buildInputs;
          nativeBuildInputs = with pkgs; [ rust-bin.nightly.latest.default ];
          preBuild = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath buildInputs}"
          '';
        };
      in
      {
        devShells.default = with pkgs; mkShell rec {
          inherit buildInputs;
          nativeBuildInputs = [
            rust-bin.nightly.latest.default
            cargo-deny
            cargo-edit
            cargo-watch
            rust-analyzer
          ];
          shellHook = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath buildInputs}"
          '';
        };

        # Build output
        packages.default = moccacino;

        # Optional: Define an app for running the built binary
        apps.default = flake-utils.lib.mkApp { drv = moccacino; };
      }
    );
}
