{ pkgs ? import <nixpkgs> {} }:

let
  tailwindcss = pkgs.stdenv.mkDerivation {
    name = "tailwindcss";
    version = "4.2.2";

    src = pkgs.fetchurl {
      url = "https://github.com/tailwindlabs/tailwindcss/releases/download/v4.2.2/tailwindcss-linux-x64";
      sha256 = "08jc0kn490s3nj7sxbbdi16iiqczapjy09gxqhz2sh3c94mlzf2a";
    };

    dontUnpack = true;
    dontStrip = true;

    installPhase = ''
      mkdir -p $out/bin
      cp $src $out/bin/tailwindcss
      chmod +x $out/bin/tailwindcss
    '';
  };

  daisyuiMjs = pkgs.fetchurl {
    url = "https://github.com/saadeghi/daisyui/releases/download/v5.5.19/daisyui.mjs";
    sha256 = "07xg75iflzh0qrfnygpfa3y6pvc23am7cbf3rhczap1wx3skzr2z";
  };
in

pkgs.mkShell {
  packages = [
    pkgs.sqlite
    pkgs.refinery-cli
    tailwindcss
  ];

  shellHook = ''
    if [ ! -f styles/daisyui.mjs ]; then
      mkdir -p styles/
      cp ${daisyuiMjs} styles/daisyui.mjs
      chmod 644 styles/daisyui.mjs
    fi
  '';
}

