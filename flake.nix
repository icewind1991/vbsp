{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.11";
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    rust-overlay,
  }:
    utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = (import nixpkgs) {
        inherit system overlays;
      };
      inherit (pkgs) lib callPackage rust-bin mkShell;
      inherit (lib.sources) sourceByRegex;

      msrv = (fromTOML (readFile ./Cargo.toml)).package.rust-version;
      inherit (builtins) fromTOML readFile;
      toolchain = rust-bin.stable.latest.default;
      msrvToolchain = rust-bin.stable."${msrv}".default;

      naersk' = callPackage naersk {
        rustc = toolchain;
        cargo = toolchain;
      };
      msrvNaersk = callPackage naersk {
        rustc = msrvToolchain;
        cargo = msrvToolchain;
      };

      src = sourceByRegex ./. ["Cargo.*" "(src|derive|benches|tests|examples|koth_bagel.*)(/.*)?"];
      nearskOpt = {
        pname = "vbsp";
        root = src;
      };
    in rec {
      packages = {
        check = naersk'.buildPackage (nearskOpt
          // {
            mode = "check";
          });
        clippy = naersk'.buildPackage (nearskOpt
          // {
            mode = "clippy";
          });
        test = naersk'.buildPackage (nearskOpt
          // {
            release = false;
            mode = "test";
          });
        msrv = msrvNaersk.buildPackage (nearskOpt
          // {
            mode = "check";
          });
      };

      devShells = let
        tools = with pkgs; [
          bacon
          cargo-edit
          cargo-outdated
          cargo-audit
          cargo-msrv
          cargo-semver-checks
          (writeShellApplication {
            name = "cargo-fuzz";
            runtimeInputs = [cargo-fuzz toolchain];
            text = ''
              # shellcheck disable=SC2068
              RUSTC_BOOTSTRAP=1 cargo-fuzz $@
            '';
          })
          (writeShellApplication {
            name = "cargo-expand";
            runtimeInputs = [cargo-expand toolchain];
            text = ''
              # shellcheck disable=SC2068
              RUSTC_BOOTSTRAP=1 cargo-expand $@
            '';
          })
        ];
      in {
        default = mkShell {
          nativeBuildInputs = [toolchain] ++ tools;
        };
        msrv = mkShell {
          nativeBuildInputs = [msrvToolchain] ++ tools;
        };
      };
    });
}
