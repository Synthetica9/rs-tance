let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  pkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
in

with pkgs.latest.rustChannels.nightly;

pkgs.mkShell {
  buildInputs = [ cargo rust ];
  # buildInputs = [ gmp mpfr libmpc m4 ];
}
