{
  description = "42arena";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };
    rust_toolchain = fenix.packages.${system}.stable.withComponents [
      "cargo"
      "rustc"
      "rust-analyzer"
      "rust-src"
      "rustfmt"
      "clippy"
    ];
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = with pkgs; [
        rust_toolchain
        just
      ];

      shellHook = ''
        echo "entering development shell"
      '';

      # LD_LIBRARY_PATH = "${pkgs.openssl.out}/lib";
    };
  };
}
