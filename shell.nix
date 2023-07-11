# For an explanation of how this file works, see: https://nix.dev/tutorials/first-steps/nix-language#shell-environment
#
# Nixpkgs can be updated with: nix-shell -p niv --run "niv update"
#

{ sources ? import ./nix/sources.nix, pkgs ? import sources.nixpkgs {} }:

pkgs.mkShell {
	buildInputs = with pkgs; [
		gtk4
		gtksourceview5
		pkg-config
		lua5_4
		rustc
		cargo
	] ++ (
		# SSL support is platform-specific
		if pkgs.lib.strings.hasSuffix "-darwin" builtins.currentSystem then with pkgs; [
			darwin.apple_sdk.frameworks.Security
			darwin.apple_sdk.frameworks.CoreServices
			openssl
		]
		else if pkgs.lib.strings.hasSuffix "-linux" builtins.currentSystem then with pkgs; [
			openssl
		]
		else []
	);
	# Needed for rust-analyzer to work with proc macros
	RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
