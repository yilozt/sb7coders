macro_rules! colorfromhex {
  ($x: expr) => {
    [(($x >> 16) & 0xFF) as f32 / 255.0,
     (($x >> 8) & 0xFF) as f32 / 255.0,
     (($x >> 0) & 0xFF) as f32 / 255.0,
     1.0]
  };
}

macro_rules! def {
  ($name: ident, $val: expr) => {
    #[allow(non_upper_case_globals)]
    pub const $name: [f32; 4] = colorfromhex!($val);
  };
}

def!(AliceBlue, 0xF0F8FF);
def!(AntiqueWhite, 0xFAEBD7);
def!(Aqua, 0x00FFFF);
def!(Aquamarine, 0x7FFFD4);
def!(Azure, 0xF0FFFF);
def!(Beige, 0xF5F5DC);
def!(Bisque, 0xFFE4C4);
def!(Black, 0x000000);
def!(BlanchedAlmond, 0xFFEBCD);
def!(Blue, 0x0000FF);
def!(BlueViolet, 0x8A2BE2);
def!(Brown, 0xA52A2A);
def!(BurlyWood, 0xDEB887);
def!(CadetBlue, 0x5F9EA0);
def!(Chartreuse, 0x7FFF00);
def!(Chocolate, 0xD2691E);
def!(Coral, 0xFF7F50);
def!(CornflowerBlue, 0x6495ED);
def!(Cornsilk, 0xFFF8DC);
def!(Crimson, 0xDC143C);
def!(Cyan, 0x00FFFF);
def!(DarkBlue, 0x00008B);
def!(DarkCyan, 0x008B8B);
def!(DarkGoldenRod, 0xB8860B);
def!(DarkGray, 0xA9A9A9);
def!(DarkGreen, 0x006400);
def!(DarkKhaki, 0xBDB76B);
def!(DarkMagenta, 0x8B008B);
def!(DarkOliveGreen, 0x556B2F);
def!(DarkOrange, 0xFF8C00);
def!(DarkOrchid, 0x9932CC);
def!(DarkRed, 0x8B0000);
def!(DarkSalmon, 0xE9967A);
def!(DarkSeaGreen, 0x8FBC8F);
def!(DarkSlateBlue, 0x483D8B);
def!(DarkSlateGray, 0x2F4F4F);
def!(DarkTurquoise, 0x00CED1);
def!(DarkViolet, 0x9400D3);
def!(DeepPink, 0xFF1493);
def!(DeepSkyBlue, 0x00BFFF);
def!(DimGray, 0x696969);
def!(DodgerBlue, 0x1E90FF);
def!(FireBrick, 0xB22222);
def!(FloralWhite, 0xFFFAF0);
def!(ForestGreen, 0x228B22);
def!(Fuchsia, 0xFF00FF);
def!(Gainsboro, 0xDCDCDC);
def!(GhostWhite, 0xF8F8FF);
def!(Gold, 0xFFD700);
def!(GoldenRod, 0xDAA520);
def!(Gray, 0x808080);
def!(Green, 0x008000);
def!(GreenYellow, 0xADFF2F);
def!(HoneyDew, 0xF0FFF0);
def!(HotPink, 0xFF69B4);
def!(IndianRed, 0xCD5C5C);
def!(Indigo, 0x4B0082);
def!(Ivory, 0xFFFFF0);
def!(Khaki, 0xF0E68C);
def!(Lavender, 0xE6E6FA);
def!(LavenderBlush, 0xFFF0F5);
def!(LawnGreen, 0x7CFC00);
def!(LemonChiffon, 0xFFFACD);
def!(LightBlue, 0xADD8E6);
def!(LightCoral, 0xF08080);
def!(LightCyan, 0xE0FFFF);
def!(LightGoldenRodYellow, 0xFAFAD2);
def!(LightGray, 0xD3D3D3);
def!(LightGreen, 0x90EE90);
def!(LightPink, 0xFFB6C1);
def!(LightSalmon, 0xFFA07A);
def!(LightSeaGreen, 0x20B2AA);
def!(LightSkyBlue, 0x87CEFA);
def!(LightSlateGray, 0x778899);
def!(LightSteelBlue, 0xB0C4DE);
def!(LightYellow, 0xFFFFE0);
def!(Lime, 0x00FF00);
def!(LimeGreen, 0x32CD32);
def!(Linen, 0xFAF0E6);
def!(Magenta, 0xFF00FF);
def!(Maroon, 0x800000);
def!(MediumAquaMarine, 0x66CDAA);
def!(MediumBlue, 0x0000CD);
def!(MediumOrchid, 0xBA55D3);
def!(MediumPurple, 0x9370DB);
def!(MediumSeaGreen, 0x3CB371);
def!(MediumSlateBlue, 0x7B68EE);
def!(MediumSpringGreen, 0x00FA9A);
def!(MediumTurquoise, 0x48D1CC);
def!(MediumVioletRed, 0xC71585);
def!(MidnightBlue, 0x191970);
def!(MintCream, 0xF5FFFA);
def!(MistyRose, 0xFFE4E1);
def!(Moccasin, 0xFFE4B5);
def!(NavajoWhite, 0xFFDEAD);
def!(Navy, 0x000080);
def!(OldLace, 0xFDF5E6);
def!(Olive, 0x808000);
def!(OliveDrab, 0x6B8E23);
def!(Orange, 0xFFA500);
def!(OrangeRed, 0xFF4500);
def!(Orchid, 0xDA70D6);
def!(PaleGoldenRod, 0xEEE8AA);
def!(PaleGreen, 0x98FB98);
def!(PaleTurquoise, 0xAFEEEE);
def!(PaleVioletRed, 0xDB7093);
def!(PapayaWhip, 0xFFEFD5);
def!(PeachPuff, 0xFFDAB9);
def!(Peru, 0xCD853F);
def!(Pink, 0xFFC0CB);
def!(Plum, 0xDDA0DD);
def!(PowderBlue, 0xB0E0E6);
def!(Purple, 0x800080);
def!(RebeccaPurple, 0x663399);
def!(Red, 0xFF0000);
def!(RosyBrown, 0xBC8F8F);
def!(RoyalBlue, 0x4169E1);
def!(SaddleBrown, 0x8B4513);
def!(Salmon, 0xFA8072);
def!(SandyBrown, 0xF4A460);
def!(SeaGreen, 0x2E8B57);
def!(SeaShell, 0xFFF5EE);
def!(Sienna, 0xA0522D);
def!(Silver, 0xC0C0C0);
def!(SkyBlue, 0x87CEEB);
def!(SlateBlue, 0x6A5ACD);
def!(SlateGray, 0x708090);
def!(Snow, 0xFFFAFA);
def!(SpringGreen, 0x00FF7F);
def!(SteelBlue, 0x4682B4);
def!(Tan, 0xD2B48C);
def!(Teal, 0x008080);
def!(Thistle, 0xD8BFD8);
def!(Tomato, 0xFF6347);
def!(Turquoise, 0x40E0D0);
def!(Violet, 0xEE82EE);
def!(Wheat, 0xF5DEB3);
def!(White, 0xFFFFFF);
def!(WhiteSmoke, 0xF5F5F5);
def!(Yellow, 0xFFFF00);
def!(YellowGreen, 0x9ACD32);

mod test {
  #[test]
  fn run() {
    assert_eq!(super::YellowGreen,
               [0x9A as f32 / 255.0,
                0xcd as f32 / 255.0,
                0x32 as f32 / 255.0,
                1.0]);
    assert_eq!(super::Violet,
               [0xee as f32 / 255.0,
                0x82 as f32 / 255.0,
                0xee as f32 / 255.0,
                1.0]);
  }
}
