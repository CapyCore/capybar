{
  description = "capybar";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      # Systems supported
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      
      # Helper function to generate packages for each system
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      
      # Function to get package for a system
      packageFor = system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };
          
          rustPlatform = pkgs.makeRustPlatform {
            cargo = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
            rustc = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
          };
          
          manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
        in
        rustPlatform.buildRustPackage {
          pname = manifest.name;
          inherit (manifest) version;

          buildInputs = with pkgs; [
            fontconfig
            libxkbcommon
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          src = pkgs.lib.cleanSource ./.;

          meta = {
              description = "Native wayland toolbar";
              homepage = "https://github.com/CapyCore/capybar";
              platforms = nixpkgs.lib.platforms.linux;
              license = nixpkgs.lib.licenses.mit;
              mainProgram = "capybar";
          };
        };
      
      # Function to build dev shell
      devShellFor = system:
        let
          pkgs = import nixpkgs { 
            inherit system; 
            overlays = [ (import rust-overlay) ];
          };
        in
        pkgs.mkShell {
          buildInputs = with pkgs; [
            (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default))
            rust-analyzer
            rustfmt
            clippy
            pkg-config
            fontconfig
            libxkbcommon
          ];
        };
    in
    {
      # Generate per-system outputs
      packages = forAllSystems (system: {
        default = packageFor system;
        capybar = packageFor system;
      });

      devShells = forAllSystems (system: {
        default = devShellFor system;
      });

      formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.nixfmt-tree);

      homeManagerModules.default = import ./nix/module.nix self;
    };
}
