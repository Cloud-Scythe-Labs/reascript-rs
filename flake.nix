{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix-core = {
      url = "github:Cloud-Scythe-Labs/nix-core";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, nix-core, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        inherit (pkgs) lib;
        pkgs = nixpkgs.legacyPackages.${system};
        toolchains = nix-core.toolchains.${system};

        rustToolchain = toolchains.mkRustToolchainFromTOML ./rust-toolchain.toml
          "sha256-opUgs6ckUQCyDxcB9Wy51pqhd0MPGHUVbwRKKPGiwZU=";
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain.fenix-pkgs;
        src = craneLib.cleanCargoSource ./.;
        commonArgs = {
          inherit src;
          strictDeps = true;
          buildInputs = rustToolchain.complete;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        individualCrateArgs = commonArgs // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
        };

        fileSetForCrate = crate: lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            ./meta
            crate
          ];
        };
        reascript-gen = craneLib.buildPackage (individualCrateArgs // {
          pname = "reascript-gen";
          cargoExtraArgs = "-p reascript-gen";
          src = fileSetForCrate ./meta/gen;
        });
        reascript-proc = craneLib.buildPackage (individualCrateArgs // {
          pname = "reascript-proc";
          cargoExtraArgs = "-p reascript-proc";
          src = fileSetForCrate ./meta/proc;
        });
      in
      {
        checks = {
          inherit reascript-gen reascript-proc;
          workspace-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          workspace-doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
          });

          # Check formatting
          workspace-fmt = craneLib.cargoFmt {
            inherit src;
          };
        };
        packages = {
          inherit reascript-gen reascript-proc;
        } // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
          my-workspace-llvm-coverage = craneLib.cargoLlvmCov (commonArgs // {
            inherit cargoArtifacts;
          });
        };
        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages = with pkgs; [
            nil
            nixpkgs-fmt
          ] ++ rustToolchain.complete;
        };
        formatter = pkgs.nixpkgs-fmt;
      }
    ) // {
      devShells.x86_64-linux.xorg-env =
        let
          pkgs = self.inputs.nixpkgs.legacyPackages.x86_64-linux;
        in
        pkgs.mkShell {
          buildInputs = with pkgs; [
            xvfb-run
            xdotool
            xclip
            gnutar
            xorg
          ];
          XDG_SESSION_TYPE="x11";
        };
    };
}
