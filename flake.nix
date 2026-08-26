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
          # Driver stack (Vulkan ICDs, DRI, EGL/GL) pinned to the SAME nixpkgs
          # revision as the rest of the shell, so driver/glibc/wayland ABIs stay
          # consistent regardless of what the host system was recently updated to.
          mesa
        ];

        # Explicit Vulkan ICD list: hardware drivers first (virtio/radeon/intel/
        # nouveau), lavapipe last as a guaranteed software-rendering fallback.
        vkDriverFiles = with pkgs.lib; concatStringsSep ":" (map (n:
          "${pkgs.mesa}/share/vulkan/icd.d/${n}") [
          "virtio_icd.x86_64.json"
          "radeon_icd.x86_64.json"
          "intel_icd.x86_64.json"
          "intel_hasvk_icd.x86_64.json"
          "nouveau_icd.x86_64.json"
          "lvp_icd.x86_64.json"
        ]);

        runtimeGlEnv = ''
          export VK_DRIVER_FILES="${vkDriverFiles}"
          export LIBGL_DRIVERS_PATH="${pkgs.mesa}/lib/dri"
        '';

        moccacino = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          inherit buildInputs;
          nativeBuildInputs = with pkgs; [
            rust-bin.nightly.latest.default
            makeWrapper
          ];
          preBuild = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath buildInputs}"
            ${runtimeGlEnv}
          '';
          # Embed the same rendering environment so `nix run`/`./result/bin/moccacino`
          # works even when the host driver stack disagrees with the closure.
          postInstall = ''
            wrapProgram "$out/bin/moccacino" \
              --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath buildInputs}" \
              --set-default VK_DRIVER_FILES "${vkDriverFiles}" \
              --set-default LIBGL_DRIVERS_PATH "${pkgs.mesa}/lib/dri"
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
            ${runtimeGlEnv}
          '';
        };

        # Build output
        packages.default = moccacino;

        # Optional: Define an app for running the built binary
        apps.default = flake-utils.lib.mkApp { drv = moccacino; };
      }
    );
}
