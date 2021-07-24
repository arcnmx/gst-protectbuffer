{ stdenv
, nix-gitignore
, gst_all_1
, gstreamer ? gst_all_1.gstreamer
, pkg-config
, rustPlatform
, buildType ? "release"
}: rustPlatform.buildRustPackage {
  pname = "gst-protectbuffer";
  version = "0.1.0";
  inherit buildType;

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ gstreamer ];

  src = nix-gitignore.gitignoreSource [ ''
    /*.nix
    /.github/
    /testdata/
  '' ] ./.;

  cargoSha256 = "0vqwll1iya9iak4vwidyxnxyiw8f7gycdj13rxqyisqbg4wrg9c3";

  libname = "libgstprotectbuffer" + stdenv.hostPlatform.extensions.sharedLibrary;

  postInstall = ''
    mkdir -p $out/lib/gstreamer-1.0
    mv $out/lib/$libname $out/lib/gstreamer-1.0/
  '';
}
