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

          rust_toolchain = fenix.packages.${system}.combine [
              fenix.packages.${system}.stable.toolchain
              fenix.packages.${system}.targets.wasm32-wasip2.stable.rust-std
            ];
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rust_toolchain
              just
              wasmtime
              nodejs_24
              # For timeout call which gets called in the backend
              coreutils
            ];

            shellHook = ''
              echo "entering development shell"
            '';
          };
        });
    };
}
