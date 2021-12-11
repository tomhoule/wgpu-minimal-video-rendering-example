{
  outputs = { nixpkgs, self }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      LD_LIBRARY_PATH = "${pkgs.vulkan-loader}/lib";
      VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
    in
    {
      defaultPackage."${system}" = pkgs.stdenv.mkDerivation {
        name = "out.y4m";
        buildInputs = [
          self.packages."${system}".wgpu-video-example
          pkgs.vulkan-loader
          pkgs.vulkan-headers
          pkgs.vulkan-tools-lunarg
          pkgs.vulkan-validation-layers
          # pkgs.spirv-headers
        ];
        src = builtins.path { name = "example-root"; path = ./.; };
        buildPhase = ''
          wgpu-video-example
        '';
        installPhase = ''
          mkdir $out
          mv out.y4m $out
        '';

        inherit LD_LIBRARY_PATH VK_LAYER_PATH;
      };

      packages."${system}".wgpu-video-example = pkgs.rustPlatform.buildRustPackage
        {
          name = "wgpu-video-example";
          src = builtins.path { name = "example-root"; path = ./.; };
          buildInputs = [
            pkgs.vulkan-loader
            # pkgs.vulkan-headers
            # pkgs.vulkan-tools-lunarg
            # pkgs.vulkan-validation-layers
            # pkgs.spirv-headers
          ];

          cargoLock = { lockFile = ./Cargo.lock; };

          inherit LD_LIBRARY_PATH VK_LAYER_PATH;
        };

      devShell."${system}" = pkgs.mkShell {
        inputsFrom = [ self.packages."${system}".wgpu-video-example ];
        packages = [ pkgs.mold ];

        inherit LD_LIBRARY_PATH VK_LAYER_PATH;
      };
    };
}
