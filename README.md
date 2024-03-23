# CP437 tools

A small collection of tools to handle CP437 files.


## Commands

### cp437-to-txt

Simply strips metadata and maps CP437 to UTF-8. Takes an optional argument to
the output file, but defaults to showing the result in the terminal.

<img src="example-txt.png" width="256" alt="TXT output"/>

### cp437-to-png

Renders the CP437 as a PNG image. Takes an optional argument to the output file,
but if not given it allows piping to other programs such as imagemagick.

<img src="example-png.png" width="256" alt="PNG output"/>


## Lib

While not intended for use as a library, it may still prove useful. Be warned
though that no guarantees are made about the stability of the API.

### cp437_tools::cp437::CP437

An array of 256 elements, mapping most of the CP437 values to UTF-8. I say most
because some have ambigous meanings and so I've taken the liberty to restrict
their use to make rendering easier.

### cp437_tools::colour::COLOURS

A list of 16 RGB values corresponding to the 4-bit colours used by CP437 files.

### cp437_tools::meta::*

A set of functions used to handle the metadata of CP437 files (aka
[SAUCE](https://www.acid.org/info/sauce/sauce.htm)).


## Development

The repo comes with a [nix flake](./flake.nix), so simply type `nix develop` and
you'll have a bash terminal with all tools needed for building this codebase.

You have an [example file](./example.ans) for simple tests (the same used for
the screenshots above), but if you need more:

```shell
$ nix build '.#test_files'
$ ls ./result/
comments.ans  large.ans  meta.ans  simple.ans
```


## Licenses

This project's code is made freely available under the [GPLv3+](./LICENSE.md)
license. The [fonts](./res/fonts) used are provided by
[int10h.org](https://int10h.org/oldschool-pc-fonts) under the
[CC-BY-SA-4.0](./res/fonts/LICENSE) license.
