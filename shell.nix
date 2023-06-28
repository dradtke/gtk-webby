{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
	buildInputs = [
		pkgs.gtk4
		pkgs.pkg-config
		pkgs.lua5_4
		pkgs.rustc
		pkgs.cargo
	] ++ (
		# SSL support is platform-specific
		if builtins.currentSystem == "x86_64-darwin" then [
			pkgs.darwin.apple_sdk.frameworks.Security
			pkgs.darwin.apple_sdk.frameworks.CoreServices
			pkgs.openssl
		]
		else if builtins.currentSystem == "x86_64-linux" then [
			pkgs.openssl
		]
		else []
	);
	RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
