{
  outputs = { nixpkgs, self }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      LD_LIBRARY_PATH = "${pkgs.vulkan-loader}/lib";
      join-frames = pkgs.writeShellScriptBin "join-frames" ''
        ffmpeg -r 60 -f image2 -i out/%05d.png -s 1792x1024 -vcodec libx264 -crf 15 -pix_fmt rgba test.mp4
      '';
    in
    {
      defaultPackage."${system}" = {
        buildInputs = [
          # pkgs.vulkan-headers
          pkgs.vulkan-loader
          pkgs.ffmpeg
          # pkgs.vulkan-tools-lunarg
          # pkgs.vulkan-validation-layers
          # pkgs.spirv-headers
        ];

        inherit LD_LIBRARY_PATH;
      };

      devShell."${system}" = pkgs.mkShell {
        inputsFrom = [ self.defaultPackage."${system}" ];
        packages = [ pkgs.mold join-frames ];

        inherit LD_LIBRARY_PATH;
      };
    };
}
