{
  inputs = {
    reaper = {
      url = "https://www.reaper.fm/files/7.x/reaper732_linux_x86_64.tar.xz";
      flake = false;
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
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
    };
  };

  outputs = { self, nixpkgs, flake-parts, crane, nix-core, fenix, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; }
      {
        systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
        perSystem = { config, self', inputs', pkgs, system, ... }:
          let
            inherit (pkgs) lib;
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
            scripts = pkgs.callPackage ./scripts/generate_reaper_plugin_functions.nix { };
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
                inherit (scripts) genReaperPluginFunctionsLua genReaperPluginFunctionsBin;
              });
            } // lib.optionalAttrs (system == "x86_64-linux") {
              reaper-latest = (pkgs.reaper.overrideAttrs {
                meta.license = "";
                src = inputs.reaper;
              });
              reaper-plugin-functions = pkgs.runCommand "reaper-plugin-functions"
                {
                  buildInputs = with pkgs; [
                    xvfb-run
                    xdotool
                    which
                    self.packages.${system}.reaper-latest
                  ];
                } ''
                mkdir -p $out/include
                xvfb-run -a bash ${scripts.genReaperPluginFunctionsBin}/bin/generate_reaper_plugin_functions.sh $(which reaper) ${scripts.genReaperPluginFunctionsLua} $out/include
              '';
            };

            devShells.default = craneLib.devShell {
              checks = self.checks.${system};
              packages = with pkgs; [
                (pkgs.callPackage ./scripts/update_reaper_flake_input.nix { })
                nil
                nixpkgs-fmt
              ] ++ rustToolchain.complete;
            };

            formatter = pkgs.nixpkgs-fmt;
          };
      };
}
