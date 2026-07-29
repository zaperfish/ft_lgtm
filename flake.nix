{
  description = "ft_lgtm";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems = f:
        nixpkgs.lib.genAttrs systems (system: f system);
    in {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };

          rust_toolchain =
            fenix.packages.${system}.stable.withComponents [
              "cargo"
              "rustc"
              "rust-analyzer"
              "rust-src"
              "rustfmt"
              "clippy"
            ];
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rust_toolchain
              just
            ];

            shellHook = ''
              echo "entering development shell"
            '';
          };
        });
    };
}
