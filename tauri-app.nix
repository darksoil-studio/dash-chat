{ self, inputs, ... }:

{
  perSystem = { inputs', pkgs, lib, self', system, ... }:
    let
      tauriConfig =
        builtins.fromJSON (builtins.readFile ./src-tauri/tauri.conf.json);
      cargoToml = builtins.fromTOML (builtins.readFile ./src-tauri/Cargo.toml);
      pname = cargoToml.package.name;
      version = tauriConfig.version;
      craneLib = (inputs.holonix.inputs.crane.mkLib pkgs).overrideToolchain
        inputs'.holonix.packages.rust;
      src = inputs.p2p-shipyard.outputs.lib.cleanTauriSource { inherit lib; }
        (craneLib.path ./.);

      ui = pkgs.stdenv.mkDerivation (finalAttrs: {
        inherit version;
        pname = "${pname}-ui";
        src =
          (inputs.tnesh-stack.outputs.lib.cleanPnpmDepsSource { inherit lib; })
          ./.;

        nativeBuildInputs = with pkgs; [ nodejs pnpm.configHook git ];
        pnpmDeps = pkgs.pnpm.fetchDeps {
          inherit (finalAttrs) version pname src;

          hash = "sha256-v8eCilIFjVCx0s/KFZZKY/Ha/Gy/6vrO8oliSI03iRc=";
          buildInputs = [ pkgs.git ];
        };
        buildPhase = ''
          runHook preBuild

          pnpm --filter=ui build

          runHook postBuild
          mkdir $out
          cp -R ui/dist $out
        '';
      });
      commonArgs = {
        inherit pname version src;

        doCheck = false;
        cargoBuildCommand =
          "cargo build --bins --release --locked --features tauri/custom-protocol,tauri/native-tls";
        cargoCheckCommand = "";
        cargoExtraArgs = "";

        buildInputs =
          inputs.p2p-shipyard.outputs.dependencies.${system}.tauriHapp.buildInputs
          ++ (lib.optionals pkgs.stdenv.isLinux [ pkgs.udev ]);

        nativeBuildInputs =
          inputs.p2p-shipyard.outputs.dependencies.${system}.tauriHapp.nativeBuildInputs;

        postPatch = ''
          mkdir -p "$TMPDIR/nix-vendor"
          cp -Lr "$cargoVendorDir" -T "$TMPDIR/nix-vendor"
          sed -i "s|$cargoVendorDir|$TMPDIR/nix-vendor/|g" "$TMPDIR/nix-vendor/config.toml"
          chmod -R +w "$TMPDIR/nix-vendor"
          cargoVendorDir="$TMPDIR/nix-vendor"
        '';
        stdenv = if pkgs.stdenv.isDarwin then
          pkgs.overrideSDK pkgs.stdenv "11.0"
        else
          pkgs.stdenv;

      };
      #cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      tauriApp = craneLib.buildPackage (commonArgs // {
        #inherit cargoArtifacts;

        cargoBuildCommand = ''
          if [ -f "src-tauri/tauri.conf.json" ]; then
            substituteInPlace src-tauri/tauri.conf.json \
              --replace-fail '"frontendDist": "../ui/dist"' '"frontendDist": "${ui}/dist"' \
              --replace-fail '"beforeBuildCommand": "pnpm -F ui build",' '"beforeBuildCommand": "",'
            cp ${
              self.outputs.packages."x86_64-linux".dash_chat_happ
            } workdir/dash-chat.happ
            cp ${
              self.outputs.packages."x86_64-linux".dash_chat_dna.hash
            } workdir/dash_chat_dna-hash
          fi
          ${commonArgs.cargoBuildCommand}'';
      });
    in rec {
      packages = {
        inherit ui;
        dash-chat = if pkgs.stdenv.isLinux then
          (pkgs.runCommandNoCC "dash-chat" {
            buildInputs = [ pkgs.makeWrapper ];

          } ''
            mkdir $out
            mkdir $out/bin
            # Because we create this ourself, by creating a wrapper
            makeWrapper ${tauriApp}/bin/dash-chat $out/bin/dash-chat \
              --set WEBKIT_DISABLE_DMABUF_RENDERER 1
          '')
        else
          tauriApp;
      };

      apps.default.program = pkgs.writeShellApplication {
        name = "${pname}-${version}";
        runtimeInputs = [ packages.dash-chat ];
        text = ''
          dash-chat
        '';
      };
    };
}
