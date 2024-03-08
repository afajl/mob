{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs = { nixpkgs.follows = "nixpkgs"; };
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile
            ./rust-toolchain.toml;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          src = craneLib.cleanCargoSource ./.;
          nativeBuildInputs = with pkgs; [ rustToolchain pkg-config ];

          buildInputs = with pkgs;
            [ ] ++ lib.optionals stdenv.isDarwin [ libiconv ];

          commonArgs = {
            pname = "mob";
            version = "0.1.0";
            inherit src buildInputs nativeBuildInputs;
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          mob =
            craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
        in
        {
          packages = {
            inherit mob;
            default = mob;
          };

          devShells.default = craneLib.devShell {
            inputsFrom = [ mob ];
            packages = [ pkgs.hyperfine ];

            shellHook = ''
              echo
              echo "👋 Hi."
            '';
          };
        }) // {
      overlays.default = final: _: {
        inherit (self.packages.${final.system}) frind;
      };
    };
}

