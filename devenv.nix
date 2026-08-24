{ pkgs, ... }:

let
  lyricsgenius = ps: ps.buildPythonPackage rec {
    pname = "lyricsgenius";
    version = "3.7.6";
    pyproject = true;

    src = ps.fetchPypi {
      inherit pname version;
      hash = "sha256-zQGrgZEz4o9RSYWmGXH8TcNXUcRSfmF+xJCROQ3cPJ4=";
    };

    nativeBuildInputs = [ ps.hatchling ];
    propagatedBuildInputs = [ ps.requests ps.beautifulsoup4 ];
    doCheck = false;
  };

  pythonEnv = pkgs.python3.withPackages (ps: [
    (lyricsgenius ps)
    ps.requests
    ps.beautifulsoup4
    ps.mutagen
    ps.xxhash
  ]);
in
{
  languages.rust = {
    enable = true;
    channel = "stable";
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
  };

  languages.javascript = {
    enable = true;
    bun.enable = true;
  };

  languages.python = {
    enable = true;
    package = pythonEnv;
  };

  packages = with pkgs; [
    pkg-config
    openssl
    cargo-deny
    git
  ];

  enterShell = ''
    export PATH="$PWD/interfaces/web-app/node_modules/.bin:$PATH"
  '';

  scripts.build.exec = ''
    ROOT=$(git rev-parse --show-toplevel)
    TARGET=""
    ARGS=()

    for arg in "$@"; do
      case "$arg" in
        libdale)    TARGET="libdale" ;;
        dale)       TARGET="dale" ;;
        actions)    TARGET="actions" ;;
        interface)  TARGET="interface" ;;
        web-app)    TARGET="interface" ;;
        *)          ARGS+=("$arg") ;;
      esac
    done

    format_code() {
      cd "$ROOT"
      cargo fmt --all
    }

    build_dale() {
      echo ""
      echo "Checking and linting code..."
      echo ""
      cd "$ROOT"
      cargo clippy -p dale -p libdale -- -D warnings
      echo ""
      echo "Running tests..."
      echo ""
      cargo test -p libdale
      echo ""
      echo "Building \`dale\` Binary..."
      echo ""
      cargo build -p dale --release "''${ARGS[@]}"
    }

    build_libdale() {
      cd "$ROOT"
      cargo clippy -p libdale -- -D warnings
      cargo test -p libdale
      cargo build -p libdale --release "''${ARGS[@]}"
    }

    build_actions() {
      echo ""
      echo "Building Actions..."
      echo ""

      cd "$ROOT"
      cargo clippy -p libactions -p get_theme -p collect -p discogs_fetch_master -p musicbrainz_search -p calculate_cover_metrics -- -D warnings
      cargo build -p get_theme -p collect -p discogs_fetch_master -p musicbrainz_search -p calculate_cover_metrics --release "''${ARGS[@]}"

      ln -sf "../target/release/get_theme" "$ROOT/actions/get_theme"
      ln -sf "../target/release/collect" "$ROOT/actions/collect"
      ln -sf "../target/release/discogs_fetch_master" "$ROOT/actions/discogs_fetch_master"
      ln -sf "../target/release/musicbrainz_search" "$ROOT/actions/musicbrainz_search"
      ln -sf "../target/release/calculate_cover_metrics" "$ROOT/actions/calculate_cover_metrics"
      ln -sf "python/get_lyrics/main.py" "$ROOT/actions/get_lyrics"
      ln -sf "python/search_cover/main.py" "$ROOT/actions/search_cover"
      ln -sf "python/embed/main.py" "$ROOT/actions/embed"
      ln -sf "python/rename/main.py" "$ROOT/actions/rename"
    }

    build_interface() {
      echo ""
      echo "Building Web App Interface..."
      echo ""

      cd "$ROOT/interfaces/web-app"
      bun run build
    }

    if [ "$TARGET" != "interface" ]; then
      format_code
    fi

    if [ "$TARGET" = "dale" ]; then
      build_dale
    elif [ "$TARGET" = "libdale" ]; then
      build_libdale
    elif [ "$TARGET" = "actions" ]; then
      build_actions
    elif [ "$TARGET" = "interface" ]; then
      build_interface
    else
      build_dale
      build_actions
      build_interface
    fi
  '';

  scripts.check.exec = ''
    ROOT=$(git rev-parse --show-toplevel)
    ARGS=()
    LINT=false

    for arg in "$@"; do
      case "$arg" in
        --lint) LINT=true ;;
        *)      ARGS+=("$arg") ;;
      esac
    done

    cd "$ROOT"
    if [ "$LINT" = true ]; then
      cargo clippy --workspace -- "''${ARGS[@]}" -D warnings
    else
      cargo check --workspace "''${ARGS[@]}"
    fi
  '';
}
