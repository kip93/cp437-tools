{
  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs/release-24.05";
    };
    rust = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-compat = {
      url = "github:edolstra/flake-compat";
    };
    # See <https://github.com/nix-systems/nix-systems>.
    systems = {
      url = "github:nix-systems/default";
    };
  };

  outputs = { self, nixpkgs, rust, systems, ... }:
    let
      systems' = import systems;
      pkgs_fun = system:
        import nixpkgs ({
          localSystem = system;
          crossSystem = system;
          overlays = [ rust.overlays.default self.overlays.default ];
        } // (self.packages.${system} or { }))
      ;

      cargo_toml = nixpkgs.lib.importTOML ./Cargo.toml;

    in
    {
      devShells = nixpkgs.lib.genAttrs systems' (system: with pkgs_fun system; {
        default = let inherit (cp437-tools) rust; in mkShell {
          inputsFrom = [ cp437-tools ];
          nativeBuildInputs = [
            gawk
            git
            grcov
            imagemagick
            (python3.withPackages (pypkgs: with pypkgs; [ selenium ]))
            rust.llvm-tools
          ];
          shellHook = ''
            export FLAKE_ROOT="$(git rev-parse --show-toplevel)"

            export CARGO_HOME="$FLAKE_ROOT/.cargo"
            export LLVM_TOOLS=${rust.llvm-tools}/lib/rustlib/${hostPlatform.config}
            export PATH="$PATH:$LLVM_TOOLS/bin"

            printf ${lib.escapeShellArg ''
              \x1B[0m
                \x1B[1m,-------.      cp437-tools\x1B[0m dev shell
                \x1B[1m| \x1B[0mCP\x1B[31m4\x1B[32m3\x1B[34m7\x1B[0;1m |\x1B[2m`.    \x1B[0;2;3m<- That right there? That's a real masterpiece!\x1B[0m
                \x1B[1m| \x1B[0;36mT\x1B[35mools\x1B[0;1m |\x1B[2m;|    \x1B[0;3mA Whole lotta failed attempts at shell functions\x1B[0m
                \x1B[1m`-------"\x1B[2m-'    \x1B[0;3mintended to simplify the development process.\x1B[0m
              \x1B[0m
                \x1B[3mLoading environment ...\x1B[0m
              \x1B[0m
            ''}

            export PATH=${buildEnv {
              name = "cp437-tools-tools";
              paths = [
                (writeShellScriptBin "update" ''
                  set -eu
                  cd "$FLAKE_ROOT"
                  cargo update --verbose
                '')
                (writeShellScriptBin "fmt" ''
                  set -eu
                  cd "$FLAKE_ROOT"
                  cargo fmt
                '')
                (writeShellScriptBin "lint" ''
                  set -eu
                  cd "$FLAKE_ROOT"
                  cargo fmt --check
                  cargo clippy
                '')
                (writeShellScriptBin "check" ''
                  set -euo pipefail

                  cd "$FLAKE_ROOT"

                  export CARGO_INCREMENTAL=0
                  export LLVM_PROFILE_FILE="$FLAKE_ROOT/target/coverage/cargo-test-%p-%m.profraw"
                  export RUSTFLAGS='-C instrument-coverage'

                  # Check
                  cargo fmt --check
                  cargo clippy
                  cargo build --all-targets --no-default-features
                  rm -rf "$FLAKE_ROOT/target/coverage" 2>/dev/null ||:
                  mkdir -p "$FLAKE_ROOT/target/coverage"
                  cargo test --no-fail-fast

                  # Coverage reports
                  grcov_wrapped() {
                    grcov "$FLAKE_ROOT" \
                      --llvm --llvm-path "$LLVM_TOOLS/bin" \
                      --binary-path "$FLAKE_ROOT/target/debug/deps" \
                      --ignore-not-existing \
                      --ignore '../*' \
                      --ignore '/*' \
                      --ignore '.cargo/registry/*' \
                      --ignore 'src/main.rs' \
                      --excl-line "#\[derive\(" \
                      --excl-start='#\[cfg\(test\)\]|#\[allow\(dead_code\)\]' \
                      --excl-stop='^\}' \
                      --excl-br-line "#\[derive\(" \
                      --excl-br-start='#\[cfg\(test\)\]|#\[allow\(dead_code\)\]' \
                      --excl-br-stop='^\}' \
                      --source-dir "$FLAKE_ROOT" \
                      --branch \
                      --no-demangle \
                      "$@"
                  }

                  ## IDE parsable report
                  grcov_wrapped -t lcov -o "$FLAKE_ROOT/target/coverage/lcov.info"

                  ## Terminal report
                  grcov_wrapped -t markdown --precision 1 \
                  | awk '
                    NR < 3 {
                      print $0
                      next
                    } {
                      print $0 | "sort -t\"|\" -nk1"
                    }
                  ' \
                  | awk -F '|' '
                    length {
                      if ($3 ~ /%/) {
                        match($3, /(\S+)%/, x)
                        if (x[1] >= 90) {
                          printf "\x1B[32m"
                        } else if (x[1] >= 50) {
                          printf "\x1B[33m"
                        } else {
                          printf "\x1B[31m"
                        }
                      }
                      print $3 "\x1B[0m" $2
                    } END {
                      match($0, /(\S+)%/, x)
                      if (x[1] >= 90) {
                        print gensub(/(\S+%)/, "\x1B[32m\\1\x1B[0m", 1)
                      } else if (x[1] >= 50) {
                        print gensub(/(\S+%)/, "\x1B[33m\\1\x1B[0m", 1)
                      } else {
                        print gensub(/(\S+%)/, "\x1B[31m\\1\x1B[0m", 1)
                      }
                    }
                  '
                '')
                (writeShellScriptBin "build" ''
                  set -eu

                  cd "$FLAKE_ROOT"
                  cargo build --all-targets --keep-going --message-format human --release --no-default-features
                  cargo build --all-targets --keep-going --message-format human --release
                  rm -rf "$FLAKE_ROOT/target/doc" 2>/dev/null ||:
                  cargo doc --message-format short --no-deps --release
                '')
                (writeShellScriptBin "run" ''
                  set -eu

                  cd "$FLAKE_ROOT"
                  RUSTFLAGS='--cap-lints warn' cargo run \
                    --message-format short --bin ${lib.escapeShellArg cargo_toml.package.name} --release -- "$@"
                '')
                (writeShellScriptBin "debug" ''
                  set -eu

                  cd "$FLAKE_ROOT"
                  RUSTFLAGS='--cap-lints warn' RUST_BACKTRACE=1 cargo run \
                    --message-format human --bin ${lib.escapeShellArg cargo_toml.package.name} -- "$@"
                '')
                (writeShellScriptBin "publish" ''
                  set -eu

                  cd "$FLAKE_ROOT"
                  cargo fmt --check
                  cargo clippy
                  cargo test
                  cargo publish --locked
                '')
              ];
            }}/bin:"$PATH"
          '';
        };
      });

      overlays.default = final: _: with final; {
        cp437-tools =
          let
            # TODO switch back to stable (https://github.com/rust-lang/rust/issues/84277)
            # rust = rust-bin.stable.${cargo_toml.package.rust-version}.minimal;
            rust = rust-bin.nightly."2024-10-18";
            rustPlatform = makeRustPlatform {
              cargo = rust.default;
              rustc = rust.default;
            };

          in
          rustPlatform.buildRustPackage {
            pname = cargo_toml.package.name;
            inherit (cargo_toml.package) version;
            src = self;
            cargoLock.lockFile = self + "/Cargo.lock";

            nativeBuildInputs = with buildPackages; [
              groff
              gzip
            ];

            preBuild = ''
              cargo fmt --check
              cargo clippy
            '';

            postInstall = ''
              OUT_DIR="$(realpath -m ./target/${hostPlatform.rust.rustcTargetSpec}/release/build/cp437-tools-*/out)"

              for manpage in $OUT_DIR/man/*.gz; do
                target="$out/share/man/man$(basename "$manpage" .gz | tail -c 2 | head -c 1)"
                mkdir -p "$target"
                cp "$manpage" "$target"
              done
            '';

            passthru = { inherit rust rustPlatform; };

            meta = with lib; {
              inherit (cargo_toml.package) description;
              mainProgram = cargo_toml.package.name;
              homepage = cargo_toml.package.homepage or cargo_toml.package.repository;
              license = with licenses; [ gpl3Plus cc-by-sa-40 ];
              maintainers = with maintainers; [ kip93 ];
            };
          }
        ;
      };

      packages = nixpkgs.lib.genAttrs systems' (system: with pkgs_fun system; rec {
        default = cp437-tools;
        inherit (pkgs) cp437-tools;

        test_files = runCommandLocal "ans_test_files"
          {
            passthru = {
              simple = test_file_simple;
              background = test_file_background;
              large = test_file_large;
              meta = test_file_meta;
              comments = test_file_comments;
              empty = test_file_empty;
              _1_byte = test_file_1_byte;
              _128_bytes = test_file_128_bytes;
              no_data = test_file_no_data;
            };
          }
          ''
            mkdir $out
            ln -sf ${test_file_simple} $out/simple.ans
            ln -sf ${test_file_background} $out/background.ans
            ln -sf ${test_file_large} $out/large.ans
            ln -sf ${test_file_meta} $out/meta.ans
            ln -sf ${test_file_comments} $out/comments.ans
            ln -sf ${test_file_empty} $out/empty.ans
            ln -sf ${test_file_1_byte} $out/1_byte.ans
            ln -sf ${test_file_128_bytes} $out/128_bytes.ans
            ln -sf ${test_file_no_data} $out/no_data.ans
          ''
        ;

        test_file = test_file_simple;
        test_file_simple = runCommandLocal "simple.ans"
          {
            meta = with lib; {
              description = "Test CP437 file";
              license = licenses.cc0;
              maintainers = with maintainers; [ kip93 ];
              platforms = platforms.all;
            };
          }
          ''
            printf "$( \
              printf '%s\n' {0..255} \
              | sed -E 's/^(10|13|26|27)$/32/g' \
              | xargs -n32 bash -c ' \
                printf "\x1B[0;3%dm" $(( $1 / 32 % 8)); \
                printf "\\\\x%02x" "''${@:1:16}"; \
                printf "\x1B[0;1;9%dm" $(( $1 / 32 % 8)); \
                printf "\\\\x%02x" "''${@:17:16}"; \
                printf '\\\\x1B[0m'; \
              ' _ \
            )" >$out
          ''
        ;

        test_file_background = runCommandLocal "background.ans"
          {
            meta = with lib; {
              description = "Test CP437 file with background colours";
              license = licenses.cc0;
              maintainers = with maintainers; [ kip93 ];
              platforms = platforms.all;
            };
          }
          ''
            printf "$( \
              printf '%s\n' {0..255} \
              | sed -E 's/^(10|13|26|27)$/32/g' \
              | xargs -n32 bash -c ' \
                printf "\x1B[0;10%d;3%dm" $(( $1 / 32 % 8)) $(( $1 / 32 % 8)); \
                printf "\\\\x%02x" "''${@:1:16}"; \
                printf "\x1B[0;4%d;1;9%dm" $(( $1 / 32 % 8)) $(( $1 / 32 % 8)); \
                printf "\\\\x%02x" "''${@:17:16}"; \
                printf '\\\\x1B[0m'; \
              ' _ \
            )" >$out
            size="$(printf %08X "$(stat -c %s $out)")"

            printf '\x1ASAUCE00' >>$out

            for _ in {1..35}; do printf ' ' >>$out; done
            for _ in {1..20}; do printf ' ' >>$out; done
            for _ in {1..20}; do printf ' ' >>$out; done

            printf 19700101 >>$out

            printf "$(printf '\\x%s' "''${size:6:2}")" >>$out
            printf "$(printf '\\x%s' "''${size:4:2}")" >>$out
            printf "$(printf '\\x%s' "''${size:2:2}")" >>$out
            printf "$(printf '\\x%s' "''${size:0:2}")" >>$out

            printf '\x1\x1' >>$out
            printf '\x20\0\x08\0\0\0\0\0' >>$out

            printf '\0' >>$out

            printf '\x01' >>$out

            printf 'IBM VGA' >>$out
            for _ in {1..15}; do printf '\0' >>$out; done
          ''
        ;

        test_file_large = runCommandLocal "large.ans"
          {
            meta = with lib; {
              description = "Large test CP437 file";
              license = licenses.cc0;
              maintainers = with maintainers; [ kip93 ];
              platforms = platforms.all;
            };
          }
          "for _ in {1..9999}; do cat ${test_file_simple} >>$out; done"
        ;

        test_file_meta = runCommandLocal "meta.ans"
          {
            meta = with lib; {
              description = "Test CP437 file with metadata";
              license = licenses.cc0;
              maintainers = with maintainers; [ kip93 ];
              platforms = platforms.all;
            };
          }
          ''
            cat ${test_file_simple} >$out

            printf '\x1ASAUCE00' >>$out

            printf TITLE >>$out
            for _ in {1..30}; do printf ' ' >>$out; done
            printf AUTHOR >>$out
            for _ in {1..14}; do printf ' ' >>$out; done
            printf GROUP >>$out
            for _ in {1..15}; do printf ' ' >>$out; done

            printf 19700101 >>$out

            size="$(printf %08X "$(stat -c %s ${test_file_simple})")"
            printf "$(printf '\\x%s' "''${size:6:2}")" >>$out
            printf "$(printf '\\x%s' "''${size:4:2}")" >>$out
            printf "$(printf '\\x%s' "''${size:2:2}")" >>$out
            printf "$(printf '\\x%s' "''${size:0:2}")" >>$out

            printf '\x1\x1' >>$out
            printf '\x20\0\x08\0\0\0\0\0' >>$out

            printf '\0' >>$out

            printf '\x01' >>$out

            printf 'IBM VGA' >>$out
            for _ in {1..15}; do printf '\0' >>$out; done
          ''
        ;

        test_file_comments = runCommandLocal "comments.ans"
          {
            meta = with lib; {
              description = "Test CP437 file with comments";
              license = licenses.cc0;
              maintainers = with maintainers; [ kip93 ];
              platforms = platforms.all;
            };
          }
          ''
            cat ${test_file_simple} >$out

            printf '\x1ACOMNT' >>$out
            printf '%-64s' Lorem ipsum dolor sit amet >>$out

            printf SAUCE00 >>$out

            printf TITLE >>$out
            for _ in {1..30}; do printf ' ' >>$out; done
            printf AUTHOR >>$out
            for _ in {1..14}; do printf ' ' >>$out; done
            printf GROUP >>$out
            for _ in {1..15}; do printf ' ' >>$out; done

            printf 19700101 >>$out

            size="$(printf %08X "$(stat -c %s ${test_file_simple})")"
            printf "$(printf '\\x%s' "''${size:6:2}")" >>$out
            printf "$(printf '\\x%s' "''${size:4:2}")" >>$out
            printf "$(printf '\\x%s' "''${size:2:2}")" >>$out
            printf "$(printf '\\x%s' "''${size:0:2}")" >>$out

            printf '\x1\x1' >>$out
            printf '\x20\0\x08\0\0\0\0\0' >>$out

            printf '\x5' >>$out

            printf '\x01' >>$out

            printf 'IBM VGA' >>$out
            for _ in {1..15}; do printf '\0' >>$out; done
          ''
        ;

        test_file_empty = runCommandLocal "empty.ans"
          {
            meta = with lib; {
              description = "Test CP437 file, without any contents";
              license = licenses.cc0;
              maintainers = with maintainers; [ kip93 ];
              platforms = platforms.all;
            };
          }
          ''
            touch $out
          ''
        ;

        test_file_1_byte = runCommandLocal "byte.ans"
          {
            meta = with lib; {
              description = "Test CP437 file, with a single byte";
              license = licenses.cc0;
              maintainers = with maintainers; [ kip93 ];
              platforms = platforms.all;
            };
          }
          ''
            printf X >$out
          ''
        ;

        test_file_128_bytes = runCommandLocal "128_bytes.ans"
          {
            meta = with lib; {
              description = "Test CP437 file, with exactly 128 bytes";
              license = licenses.cc0;
              maintainers = with maintainers; [ kip93 ];
              platforms = platforms.all;
            };
          }
          ''
            for _ in {0..127}; do
              printf X >>$out
            done
          ''
        ;

        test_file_no_data = runCommandLocal "no_data.ans"
          {
            meta = with lib; {
              description = "Test CP437 file with metadata, but no data";
              license = licenses.cc0;
              maintainers = with maintainers; [ kip93 ];
              platforms = platforms.all;
            };
          }
          ''
            printf '\x1ASAUCE00' >>$out

            printf TITLE >>$out
            for _ in {1..30}; do printf ' ' >>$out; done
            printf AUTHOR >>$out
            for _ in {1..14}; do printf ' ' >>$out; done
            printf GROUP >>$out
            for _ in {1..15}; do printf ' ' >>$out; done

            printf 19700101 >>$out

            printf "$(printf '\\x%s' 0)" >>$out
            printf "$(printf '\\x%s' 0)" >>$out
            printf "$(printf '\\x%s' 0)" >>$out
            printf "$(printf '\\x%s' 0)" >>$out

            printf '\x1\x1' >>$out
            printf '\x20\0\x08\0\0\0\0\0' >>$out

            printf '\0' >>$out

            printf '\x01' >>$out

            printf 'IBM VGA' >>$out
            for _ in {1..15}; do printf '\0' >>$out; done
          ''
        ;
      });
    }
  ;

  nixConfig = {
    extra-substituters = [
      "https://cp437-tools.cachix.org"
    ];
    extra-trusted-public-keys = [
      "cp437-tools.cachix.org-1:1edoysGhERaj+swHGZK44RoqhfnX/V8gTLu9Rh7Ljb4="
    ];
  };
}
