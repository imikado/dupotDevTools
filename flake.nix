{
  description = "Dupot Dev Tools — Rust GTK4/Libadwaita application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        nativeBuildInputs = with pkgs; [
          pkg-config
          wrapGAppsHook4
          copyDesktopItems
        ];

        buildInputs = with pkgs; [
          glib
          gtk4
          libadwaita
          gtksourceview5
        ];

        dupot-dev-tools = pkgs.rustPlatform.buildRustPackage {
          pname = "dupot-dev-tools";
          version = "1.0.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          inherit nativeBuildInputs buildInputs;

          desktopItems = [
            (pkgs.makeDesktopItem {
              name = "org.dupot.devtools";
              desktopName = "Dupot Dev Tools";
              comment = "The developer's Swiss Army knife";
              exec = "dupot-dev-tools";
              icon = "org.dupot.devtools";
              categories = [ "Development" "Utility" ];
            })
          ];

          postInstall = ''
            install -Dm644 src/assets/logo512.png \
              $out/share/icons/hicolor/512x512/apps/org.dupot.devtools.png
            install -Dm644 src/assets/logo192.png \
              $out/share/icons/hicolor/192x192/apps/org.dupot.devtools.png
          '';

          meta = {
            description = "The developer's Swiss Army knife for the Linux desktop";
            license = pkgs.lib.licenses.lgpl21Only;
            mainProgram = "dupot-dev-tools";
          };
        };
      in
      {
        packages.default = dupot-dev-tools;

        apps.default = flake-utils.lib.mkApp { drv = dupot-dev-tools; };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ dupot-dev-tools ];
          packages = with pkgs; [
            cargo
            rustc
            clippy
            rustfmt
            rust-analyzer
          ];

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

          shellHook = ''
            export XDG_DATA_DIRS="${pkgs.gtksourceview5}/share:${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}:${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:''${XDG_DATA_DIRS:-}"
            echo "Rust GTK4/Libadwaita development environment" >&2
            rustc --version >&2
          '';
        };
      }
    );
}
