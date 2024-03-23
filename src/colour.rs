//! ANSI colours

/// A list of 16 RGB values corresponding to the 4-bit colours used by CP437
/// files
///
/// List of colours:
///
///   * Dark
///
///     0.  Black
///     1.  Red
///     2.  Green
///     3.  Yellow
///     4.  Blue
///     5.  Magenta
///     6.  Cyan
///     7.  White
///
///   * Light
///
///     8.  Black
///     9.  Red
///     10. Green
///     11. Yellow
///     12. Blue
///     13. Magenta
///     14. Cyan
///     15. White
///
#[rustfmt::skip]
pub static COLOURS: &[[u8; 3]] = &[
  // DARK
  [0x00, 0x00, 0x00], // BLACK
  [0xAB, 0x00, 0x00], // RED
  [0x00, 0xAB, 0x00], // GREEN
  [0xAB, 0x57, 0x00], // YELLOW
  [0x00, 0x00, 0xAB], // BLUE
  [0xAB, 0x00, 0xAB], // MAGENTA
  [0x00, 0xAB, 0xAB], // CYAN
  [0xAB, 0xAB, 0xAB], // WHITE

  // BRIGHT
  [0x57, 0x57, 0x57], // BLACK
  [0xFF, 0x57, 0x57], // RED
  [0x57, 0xFF, 0x57], // GREEN
  [0xFF, 0xFF, 0x57], // YELLOW
  [0x57, 0x57, 0xFF], // BLUE
  [0xFF, 0x57, 0xFF], // MAGENTA
  [0x57, 0xFF, 0xFF], // CYAN
  [0xFF, 0xFF, 0xFF], // WHITE
];
