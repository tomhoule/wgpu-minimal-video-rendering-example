{
  outputs = { nixpkgs, self }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      LD_LIBRARY_PATH = "${pkgs.vulkan-loader}/lib";
    in
    {
      defaultPackage."${system}" = {
        buildInputs = [
          # pkgs.vulkan-headers
          pkgs.vulkan-loader
          # pkgs.vulkan-tools-lunarg
          # pkgs.vulkan-validation-layers
          # pkgs.spirv-headers
        ];

        inherit LD_LIBRARY_PATH;
      };

      devShell."${system}" = pkgs.mkShell {
        inputsFrom = [ self.defaultPackage."${system}" ];
        packages = [ pkgs.mold ];

        inherit LD_LIBRARY_PATH;
      };
    };
}
