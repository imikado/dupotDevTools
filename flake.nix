{
  description = "Dupot Dev Tools — Rust GTK4/Libadwaita application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # Cross-compilation to Windows (build_windows_cross.sh): Rust with the
        # windows-gnu std, and the MinGW-w64 GCC/binutils (UCRT runtime, like
        # the MSYS2 ucrt64 GTK packages) used as linker and for windres/objdump.
        mingw = pkgs.pkgsCross.ucrt64.buildPackages;
        rustWindows = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "x86_64-pc-windows-gnu" ];
        };

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
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          inherit nativeBuildInputs buildInputs;

          # Shown in the About dialog (the Nix sandbox has no .git).
          env = {
            DUPOT_BUILD_PACKAGE = "nix";
            DUPOT_BUILD_DATE =
              let d = self.lastModifiedDate or "";
              in if d == "" then "" else
                "${builtins.substring 0 4 d}-${builtins.substring 4 2 d}-${builtins.substring 6 2 d}";
          };

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

        # update_screenshots.sh: runs the app's scripted demo in a headless
        # Wayland compositor and captures it.
        devShells.screenshots = pkgs.mkShell {
          inputsFrom = [ dupot-dev-tools ];
          packages = with pkgs; [
            cargo
            rustc
            sway
            grim
            wf-recorder
            ffmpeg
            sqlite
            dbus
          ];

          # Fixed fonts, so the screenshots don't depend on the machine.
          FONTCONFIG_FILE = pkgs.makeFontsConf {
            fontDirectories = with pkgs; [ adwaita-fonts cantarell-fonts dejavu_fonts ];
          };

          shellHook = ''
            export XDG_DATA_DIRS="${pkgs.adwaita-icon-theme}/share:${pkgs.hicolor-icon-theme}/share:${pkgs.gtksourceview5}/share:${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}:${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:''${XDG_DATA_DIRS:-}"
            echo "Screenshot environment (headless sway, grim, wf-recorder, ffmpeg)" >&2
          '';
        };

        devShells.windows = pkgs.mkShell {
          packages = [
            rustWindows
            mingw.gcc
            mingw.binutils
            pkgs.pkg-config
            pkgs.glib.dev # glib-compile-schemas
            pkgs.gtk4.dev # gtk4-update-icon-cache
            pkgs.python3
            pkgs.zstd
            pkgs.curl
            pkgs.zip
            # Runs gdk-pixbuf-query-loaders.exe, and lets you try the result:
            # wine build-windows-cross/DupotDevTools/bin/dupot-dev-tools.exe
            pkgs.wineWow64Packages.stable
          ];

          CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "x86_64-w64-mingw32-gcc";

          shellHook = ''
            echo "Windows cross-compilation environment (x86_64-pc-windows-gnu)" >&2
          '';
        };
      }
    );
}
