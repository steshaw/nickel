{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
    let
        pkgs = nixpkgs.legacyPackages.${system};
        buildPdf = { isDevShell ? false }:
          pkgs.stdenv.mkDerivation {
            name = "nickel-type-system-specification";
            src = if isDevShell then null else ./.;
            buildInputs = with pkgs; [
              # biber
              pdftk
              zip
              ott
              (texlive.combine {
                inherit (texlive)
                # biblatex
                collection-fontsrecommended
                # comment
                # fontaxes
                latexmk
                # listings
                # lm
                scheme-small
                # ucs
                wasy cm-super unicode-math lm-math capt-of
                mathpartir
                # appendix
                supertabular;
              })
            ];

            installPhase = ''
              mkdir -p $out
              mv type-system.pdf $out
            '';
          };

      in rec {
        defaultPackage = packages.buildPdf;
        packages.buildPdf = buildPdf {};
        devShell = buildPdf {isDevShell = true;};
      });
}
