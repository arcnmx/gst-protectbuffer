{ pkgs ? import <nixpkgs> { } }: let
  package = import ./default.nix { inherit pkgs; };
in package.shell
