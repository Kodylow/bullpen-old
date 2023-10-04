{ pkgs }: {
	deps = [
        pkgs.openssl
        pkgs.pkg-config
		pkgs.rustc
		pkgs.rustfmt
		pkgs.cargo
		pkgs.cargo-edit
    pkgs.rust-analyzer
	];
}