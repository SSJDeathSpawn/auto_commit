{
  description = "Rust development shell";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-25.11";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { 
    self, 
    nixpkgs,
    fenix,
  }: 
  let 
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [fenix.overlays.default];
    };
    rust = fenix.packages.${system}.stable.withComponents [
      "cargo"
      "clippy"
      "rust-src"
      "rustc"
      "rustfmt"
      "rust-analyzer"
    ];
  in
  {

    devShells.x86_64-linux.default = pkgs.mkShell {
      nativeBuildInputs = [ pkgs.pkg-config ];
      buildInputs = [ pkgs.openssl ];
      packages = [
          rust
      ];
    };

    packages.x86_64-linux.default = self.packages.x86_64-linux.auto_commit;

    packages.x86_64-linux.auto_commit = pkgs.rustPlatform.buildRustPackage (finalAttrs: {
      pname = "auto_commit";
      version = "0.0.1";

      src = ./.;
      nativeBuildInputs = with pkgs; [
        pkg-config
      ];
      buildInputs = with pkgs; [
          openssl
      ];
      cargoHash = "sha256-MWKiYrlf6u5EPa+kQW4USpEsnyM3BZh07lBb40v3T3A=";
    });
  };
}
