{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
	buildInputs = [
		pkgs.gtk4
		pkgs.pkg-config
		pkgs.lua5_4
		pkgs.rustc
		pkgs.cargo
		# For SSL support
		pkgs.darwin.apple_sdk.frameworks.Security
		pkgs.darwin.apple_sdk.frameworks.CoreServices
		pkgs.openssl
	];
	RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
