{}:
let
  rev = "1c3a28d84f970e7774af04372ade06399add182e";
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
  pkgs = import nixpkgs { };
  dfx-env = import (builtins.fetchTarball "https://github.com/ninegua/ic-nix/releases/download/20240610/dfx-env.tar.gz") { version = "20240610"; inherit pkgs; };
in
dfx-env.overrideAttrs (old: {
  nativeBuildInputs = with pkgs; old.nativeBuildInputs ++
    [
      cargo
      rustc
      pkg-config
      openssl
      protobuf
      cmake
      cachix
      killall
      jq
      coreutils
      bc
      python3Full
    ] ++ (if pkgs.stdenv.isDarwin then [
      darwin.apple_sdk.frameworks.Foundation
      pkgs.darwin.libiconv
    ] else []);
  shellHook = ''
    # Store the original PATH
    export ORIGINAL_PATH="$PATH"

    # Remove .cargo/bin from PATH
    export PATH=$(echo "$PATH" | tr ':' '\n' | grep -v "$HOME/.cargo/bin" | tr '\n' ':' | sed 's/:$//')

    # Ensure Nix-provided cargo and rustc are first in PATH
    export PATH="${pkgs.cargo}/bin:${pkgs.rustc}/bin:$PATH"

    # Install candid-extractor if not already in Nix environment
    if ! command -v candid-extractor &> /dev/null; then
      echo "Installing candid-extractor..."
      cargo install --force candid-extractor
    fi

    # Function to clean up on exit
    cleanup() {
      # Restore the original PATH
      export PATH="$ORIGINAL_PATH"
    }

    # Set up trap to call cleanup function on exit
    trap cleanup EXIT
  '';
})
