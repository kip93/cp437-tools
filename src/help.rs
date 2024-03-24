use indoc::indoc;

pub fn print() {
    eprintln!(indoc! {"
        \x1B[0m
        \x1B[1mNAME\x1B[0m
        ----------------------------------------------------------------
          cp437-tools - \x1B[3mA small collection of tools for working with
                        CP437 and ANSI/ASCII art.\x1B[0m

        \x1B[1mSYNOPSIS\x1B[0m
        ----------------------------------------------------------------
          cp437-tools \x1B[4mCOMMAND\x1B[0m [ \x1B[4mOPTIONS\x1B[0m ]

        \x1B[1mCOMMANDS\x1B[0m
        ----------------------------------------------------------------
          - help
              \x1B[3mShow this help text and exit.\x1B[0m
          - read-meta \x1B[4mINPUT\x1B[0m
              \x1B[3mShows the file's metadata.\x1B[0m
          - remove-meta \x1B[4mINPUT\x1B[0m [ \x1B[4mOUTPUT\x1B[0m ]
              \x1B[3mShows the file's metadata.\x1B[0m
          - to-png \x1B[4mINPUT\x1B[0m [ \x1B[4mOUTPUT\x1B[0m ]
              \x1B[3mRender file as PNG.\x1B[0m
          - to-txt \x1B[4mINPUT\x1B[0m [ \x1B[4mOUTPUT\x1B[0m ]
              \x1B[3mTranspile file as UTF-8.\x1B[0m

        \x1B[1mLICENSE\x1B[0m
        ----------------------------------------------------------------
          License GPLv3+: GNU GPL version 3 or later
          <https://gnu.org/licenses/gpl.html>

          This is free software: you are free to change and redistribute
          it. There is NO WARRANTY, to the extent permitted by law.
    "});
}
