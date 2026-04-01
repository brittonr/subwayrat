{
  description = "subwayrat — TUI widget library for ratatui";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    unit2nix.url = "github:brittonr/unit2nix";
    flake-utils.url = "github:numtide/flake-utils";
    ratcore = {
      url = "github:brittonr/ratcore";
      flake = false;
    };
  };

  outputs =
    {
      nixpkgs,
      unit2nix,
      flake-utils,
      ratcore,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Combine subwayrat source with ratcore (path dep at ../ratcore)
        src = pkgs.runCommand "subwayrat-src" {} ''
          cp -r ${./.} $out
          chmod -R u+w $out
          cp -r ${ratcore} $out/ratcore
          sed -i 's|path = "../ratcore"|path = "ratcore"|' $out/Cargo.toml
        '';

        ws = unit2nix.lib.${system}.buildFromUnitGraphAuto {
          inherit pkgs src;
        };
      in
      {
        packages = {
          default = ws.workspaceMembers."showcase".build;
          showcase = ws.workspaceMembers."showcase".build;
          all = ws.allWorkspaceMembers;
        } // builtins.mapAttrs (_: m: m.build) ws.workspaceMembers;

        devShells.default = pkgs.mkShell {
          inputsFrom = [ ws.workspaceMembers."showcase".build ];
          packages = [
            pkgs.cargo
            pkgs.rustc
            pkgs.rust-analyzer
            pkgs.clippy
            pkgs.rustfmt
            unit2nix.packages.${system}.unit2nix
          ];
        };
      }
    );
}
