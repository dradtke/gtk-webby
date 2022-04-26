{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
	buildInputs = [
		pkgs.gtk4
		pkgs.pkg-config
		pkgs.rustc
		pkgs.cargo
		pkgs.lua5_4
	];
}
