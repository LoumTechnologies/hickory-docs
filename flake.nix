{
  # Cell images: what software exists inside a Hickory Docs execution cell
  # when it runs on Cloud Canopy.
  #
  # Canopy sandboxes boot NixOS guests built from nixosConfigurations, so a
  # cell's toolchain is defined here the same reproducible way a canopy node
  # is defined — no Dockerfiles, no mutable base images. Each package below
  # produces a Firecracker-bootable image directory (vmlinux + initrd +
  # rootfs.ext4 + boot-args) whose immutable /nix/store path is what a
  # tenant ledger allowlists and what CANOPY_IMAGE_MAP maps `image=` refs to.
  #
  # Build on a machine with nix (e.g. colo-1's builder), typically with the
  # local canopy checkout: nix build .#cell-datasci \
  #   --override-input cloud-canopy path:/home/loumtech/Documents/src/cloud-canopy
  # Then declare the resulting store path in the ledger and map it, e.g.
  #   CANOPY_IMAGE_MAP={"python:3.12":"<store path of cell-python>", ...}
  #
  # NOT built in this repo's CI (needs nix + KVM-capable builder). The
  # LocalExecutor ignores images entirely; these exist for canopy mode.

  description = "Hickory Docs — cell images for Cloud Canopy execution";

  inputs = {
    cloud-canopy.url = "github:LoumTechnologies/cloud-canopy";
  };

  outputs = { self, cloud-canopy }:
    let
      system = "x86_64-linux";
      nixpkgs = cloud-canopy.inputs.nixpkgs;
      pkgs = nixpkgs.legacyPackages.${system};
      lib = nixpkgs.lib;

      # Extend canopy's hardened sandbox guest instead of redefining it:
      # same egress floor, same step-runner contract, more packages.
      baseGuest = cloud-canopy.nixosConfigurations.sandbox-guest;

      mkCellImage = { name, packages, spareMib ? 1024 }:
        import "${cloud-canopy}/nix/sandbox-image.nix" {
          inherit pkgs lib;
          name = "hickory-cell-${name}";
          inherit spareMib;
          guest = baseGuest.extendModules {
            modules = [{ environment.systemPackages = packages; }];
          };
        };

      rWithGgplot = pkgs.rWrapper.override {
        packages = with pkgs.rPackages; [ ggplot2 svglite ];
      };
      pythonData = pkgs.python3.withPackages (p: [ p.polars p.duckdb ]);
    in
    {
      packages.${system} = rec {
        # Shell/coreutils cell — the mapping target for `alpine:*`-style refs.
        cell-shell = mkCellImage {
          name = "shell";
          packages = with pkgs; [ coreutils gnused gawk gnugrep findutils ];
          spareMib = 512;
        };
        # Python data cell — mapping target for `python:*` refs.
        cell-python = mkCellImage {
          name = "python";
          packages = [ pythonData pkgs.uv ];
        };
        # The grand-tour cell: R + ggplot2, DuckDB CLI, Python + polars.
        cell-datasci = mkCellImage {
          name = "datasci";
          packages = [ rWithGgplot pkgs.duckdb pythonData ] ++ (with pkgs; [
            coreutils
            gnused
            gawk
          ]);
          spareMib = 2048;
        };
        default = cell-datasci;
      };
    };
}
