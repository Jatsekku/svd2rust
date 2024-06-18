{
  description = "Nix-flake for rust development";

  inputs = { nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05"; };

  outputs = { self, nixpkgs, ... }:
    let system = "x86_64-linux";
    in {
      formatter."${system}" = nixpkgs.legacyPackages."${system}".nixfmt-classic;
      devShells."${system}".default =
        let pkgs = import nixpkgs { inherit system; };
        in pkgs.mkShell {
          name = "rust-shell";
          packages = with pkgs; [ 
            cargo
            rustup
            rustc
          ];
        };
    };
}
