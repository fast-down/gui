# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      stdenv = pkgs.stdenv;
      lib = pkgs.lib;
      fhs = pkgs.buildFHSEnv {
        name = "fhs-shell";
        targetPkgs = pkgs: with pkgs; [
            gcc
            libayatana-appindicator
            libappindicator-gtk2
            libz
            glibc
            glibc.dev
            clang
            libclang
            openssl
            openssl.dev
            pango.dev
            atk.dev
            cairo.dev
            gtk3.dev
            glib.dev
            harfbuzz.dev
            gdk-pixbuf.dev
            webkitgtk_6_0.dev
            libsoup_3.dev
            libz.dev
            sqlite.dev
            zlib-ng.dev
            boringssl
            boringssl.dev
            nghttp2
            nghttp3
            nghttp3.dev
            ngtcp2
            curl
            curl.dev
            pkg-config
            wayland
            wayland-utils
            cargo
            rustc
            bun
             (pkgs.writeShellScriptBin "init-env" ''
               export PKG_CONFIG_PATH=${lib.strings.concatStringsSep ":" [
                  "${pkgs.openssl.dev}/lib/pkgconfig"
                  "${pkgs.pango.dev}/lib/pkgconfig"
                  "${pkgs.atk.dev}/lib/pkgconfig"
                  "${pkgs.cairo.dev}/lib/pkgconfig"
                  "${pkgs.gtk3.dev}/lib/pkgconfig"
                  "${pkgs.glib.dev}/lib/pkgconfig"
                  "${pkgs.harfbuzz.dev}/lib/pkgconfig"
                  "${pkgs.gdk-pixbuf.dev}/lib/pkgconfig"
                  "${pkgs.libsoup_3.dev}/lib/pkgconfig"
                  "${pkgs.webkitgtk_6_0.dev}/lib/pkgconfig"
                  "${pkgs.libz.dev}/lib/pkgconfig"
                  "${pkgs.zlib.dev}/lib/pkgconfig"
                  "${pkgs.zlib.dev}/lib/pkgconfig"
                  "${pkgs.zlib-ng.dev}/lib/pkgconfig"
                  "${pkgs.sqlite.dev}/lib/pkgconfig"
                ]}
                exec ${pkgs.bash}/bin/bash
             '')

        ];
        runScript = "init-env";
      };
    in
      {
        devShells.${system}.default = fhs.env;
      };
}