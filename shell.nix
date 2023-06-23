{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
	buildInputs = [
		pkgs.gtk4
		pkgs.pkg-config
		pkgs.cargo
		pkgs.lua5_4
		pkgs.rustc
		pkgs.cargo
	];
	RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
