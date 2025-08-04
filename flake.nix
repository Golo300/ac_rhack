{
  description = "Flake for building the hack";

  inputs = {
    nixpkgs.follows = "system-flake/nixpkgs";
    system-flake.url = "/home/timl/dotfiles";
  };

  outputs = { self, nixpkgs, ... }:
    let
      system = "x86_64-linux"; 
      pkgs = import nixpkgs { system = system; };
    in
    {
        devShells.${system}.default = pkgs.mkShell {
          name = "kernel-module-dev";

          buildInputs = with pkgs; [
            cargo
            libGL
            SDL_image
          ];

          shellHook = ''
            export LIBCLANG_PATH=${pkgs.llvmPackages.libclang.lib}/lib
          '';
        };

};
}
