{ pkgs ? import <nixpkgs> { } }: with pkgs; with gst_all_1; let
  gst-protectbuffer = pkgs.callPackage ./derivation.nix { };
  gst-protectbuffer-debug = gst-protectbuffer.override { buildType = "debug"; };
  testCommand = writeShellScriptBin "runtest" ''
    ${./testdata/runtest.sh} ${./testdata/input.jpg}
  '';
  testshell = mkShell {
    GST_PLUGIN_SYSTEM_PATH_1_0 = lib.makeSearchPath "lib/gstreamer-1.0" [
      gstreamer
      gst-plugins-base
      gst-protectbuffer-debug
    ];
    GST_DEBUG = "DEBUG"; # WARNING/LOG/TRACE?
    nativeBuildInputs = [
      testCommand
      gstreamer.dev
    ];
  };
  rustShell = shells.rust.nightly.overrideAttrs (old: {
    nativeBuildInputs = old.nativeBuildInputs or [ ] ++ gst-protectbuffer.nativeBuildInputs;
    buildInputs = old.buildInputs or [ ] ++ gst-protectbuffer.buildInputs;
  });
in gst-protectbuffer // {
  inherit testshell;
  shell = if pkgs ? shells.rust then rustShell else gst-protectbuffer;
  debug = gst-protectbuffer-debug;
}
