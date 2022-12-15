{
  description = "Developement environment for Nix users";

  inputs = {
    devshell.url = "github:numtide/devshell";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, flake-utils, devshell, nixpkgs, fenix }:
    flake-utils.lib.eachDefaultSystem (system: {
      devShell =
        let pkgs = import nixpkgs {
          inherit system;

          overlays = [ devshell.overlay fenix.overlay ];
        };
        in
        pkgs.devshell.mkShell {
          imports = [ "${pkgs.devshell.extraModulesDir}/language/c.nix" ];
          commands = [
            {
              package = pkgs.devshell.cli;
              help = "Per project developer environments";
            }
            {
              category = "project commands";
              name = "run";
              command = "cargo r";
              help = "runs this project";
            }
            {
              category = "project commands";
              name = "build";
              command = "cargo b";
              help = "builds this project";
            }
          ];
          devshell.packages = with pkgs; [
            (pkgs.fenix.complete.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
            ])
            rust-analyzer-nightly
            gitflow
            pkgconfig
          ];
          language.c.libraries = with pkgs; [
          ];
          language.c.includes = with pkgs; [
            cairo
            glib
            gdk-pixbuf
            harfbuzz
            gobject-introspection
            graphene
            gtk4
            libadwaita
            pango
            appstream-glib
          ];
        };
    });
}
