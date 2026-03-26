{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  packages = [
    pkgs.sqlite
  ];
  buildInputs = [
    (pkgs.stdenv.mkDerivation {
      name = "tailwindcss-extra";
      version = "1.7.27";
      
      src = pkgs.fetchurl {
        url = "https://github.com/dobicinaitis/tailwind-cli-extra/releases/download/v1.7.27/tailwindcss-extra-linux-x64";
        sha256 = "1r0fxlkwldbzwwx7aphifyms86qbapsp1d684p4m33s1shdfrwz5";
      };

      dontUnpack = true;
      dontStrip = true;

      installPhase = ''
        mkdir -p $out/bin
        cp $src $out/bin/tailwindcss-extra
        chmod +x $out/bin/tailwindcss-extra
      '';
    })
  ];
}
