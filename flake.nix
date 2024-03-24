{
  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs/release-23.11";
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

      cargo = nixpkgs.lib.importTOML ./Cargo.toml;

    in
    {
      devShells = nixpkgs.lib.genAttrs systems' (system: with pkgs_fun system; {
        default = mkShell {
          nativeBuildInputs = [
            rust-bin.stable.${cargo.package.rust-version}.default
          ];
          shellHook = ''
            export CARGO_HOME="$PWD/.cargo"

            update() { cargo update; }
            fmt()    { cargo fmt; }
            lint()   { cargo clippy; }
            check()  { cargo fmt --check && cargo clippy; }

            build() {
              cargo build --all-targets --message-format short --release;
            }
            build_debug() {
              cargo build --all-targets --message-format human;
            }
            run() {
              cargo run --message-format short --bin ${lib.escapeShellArg cargo.package.name} --release -- "$@";
            }
            run_debug() {
              RUST_BACKTRACE=1 cargo run --message-format short --bin ${lib.escapeShellArg cargo.package.name} -- "$@";
            }
            doc() {
              cargo doc --message-format short --no-deps
            }
            doc_open() {
              firefox "file://$PWD/target/doc"/${lib.escapeShellArg cargo.package.name}/index.html
            }
          '';
        };
      });

      overlays.default = final: _: with final; {
        cp437-tools =
          let
            rustPlatform = makeRustPlatform {
              cargo = rust-bin.stable.${cargo.package.rust-version}.minimal;
              rustc = rust-bin.stable.${cargo.package.rust-version}.minimal;
            };

          in
          rustPlatform.buildRustPackage {
            pname = cargo.package.name;
            inherit (cargo.package) version;
            src = self;
            cargoLock.lockFile = self + "/Cargo.lock";

            meta = with lib; {
              inherit (cargo.package) description;
              mainProgram = cargo.package.name;
              homepage = cargo.package.homepage or cargo.package.repository;
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
            };
          }
          ''
            mkdir $out
            ln -sf ${test_file_simple} $out/simple.ans
            ln -sf ${test_file_background} $out/background.ans
            ln -sf ${test_file_large} $out/large.ans
            ln -sf ${test_file_meta} $out/meta.ans
            ln -sf ${test_file_comments} $out/comments.ans
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
              printf '%s\n' $(seq 0 255) \
              | sed -E 's/^(10|13|26|27)$/32/g' \
              | xargs -n32 bash -c ' \
                printf "\x1B[0;3%dm" $(( $1 / 32 % 8)); \
                printf "\\\\x%02x" "''${@:1:16}"; \
                printf "\x1B[0;1;9%dm" $(( $1 / 32 % 8)); \
                printf "\\\\x%02x" "''${@:17:16}" 13 10; \
              ' _ \
              && printf '\x1B[0m' \
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
              printf '%s\n' $(seq 0 255) \
              | sed -E 's/^(10|13|26|27)$/32/g' \
              | xargs -n32 bash -c ' \
                printf "\x1B[0;10%d;3%dm" $(( $1 / 32 % 8)) $(( $1 / 32 % 8)); \
                printf "\\\\x%02x" "''${@:1:16}"; \
                printf "\x1B[0;4%d;1;9%dm" $(( $1 / 32 % 8)) $(( $1 / 32 % 8)); \
                printf "\\\\x%02x" "''${@:17:16}"; \
                printf '\\\\x1B[0m\\\\x0D\\\\x0A'; \
              ' _ \
            )" >$out
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
      });
    }
  ;
}
