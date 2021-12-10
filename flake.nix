{
  outputs = { nixpkgs, self }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      defaultPackage."${system}" = {
        buildInputs = [
        ];
      };

      devShell."${system}" = pkgs.mkShell {
        inputsFrom = [ self.defaultPackage."${system}" ];
      };
    };
}
