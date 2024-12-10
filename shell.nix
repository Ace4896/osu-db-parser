let
  rust_overlay = fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-24.05";
  pkgs = import nixpkgs { config = {}; overlays = [ (import rust_overlay) ]; };
in
pkgs.mkShellNoCC {
  buildInputs = with pkgs; [
    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
  ];

  packages = with pkgs; [
    # Trunk for WASM Deployment
    trunk
  ];
}
