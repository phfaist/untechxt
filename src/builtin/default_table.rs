
use super::{StaticTable, NeedsProfile};

use super::statictable_def::{compile_static_table,};


const ENTRIES : [(char, &str, NeedsProfile)] = &[
    ('\u{0022}', r#"''"#, BUILTINS), // QUOTATION MARK
    ('\u{0023}', r#"\#"#, BUILTINS), // NUMBER SIGN
    ('\u{0024}', r#"\$"#, BUILTINS), // DOLLAR SIGN
    ('\u{0025}', r#"\%"#, BUILTINS), // PERCENT SIGN
    ('\u{0026}', r#"\&"#, BUILTINS), // AMPERSAND
    ('\u{003C}', r#"\ensuremath{<}"#, BUILTINS), // LESS-THAN SIGN
    ('\u{003E}', r#"\ensuremath{>}"#, BUILTINS), // GREATER-THAN SIGN
    ('\u{005C}', r#"\textbackslash"#, BUILTINS), // REVERSE SOLIDUS
    ('\u{005E}', r#"\textasciicircum"#, BUILTINS), // CIRCUMFLEX ACCENT
    ('\u{005F}', r#"\_"#, BUILTINS), // LOW LINE
    ('\u{007B}', r#"\{"#, BUILTINS), // LEFT CURLY BRACKET
    ('\u{007D}', r#"\}"#, BUILTINS), // RIGHT CURLY BRACKET
    ('\u{007E}', r#"\textasciitilde"#, BUILTINS), // TILDE
    ('\u{00A0}', r#"~"#, BUILTINS), // NO-BREAK SPACE
    ('\u{00A1}', r#"\textexclamdown"#, BUILTINS), // INVERTED EXCLAMATION MARK
    ('\u{00A2}', r#"\textcent"#, BUILTINS), // CENT SIGN
    ('\u{00A3}', r#"\textsterling"#, BUILTINS), // POUND SIGN
    ('\u{00A4}', r#"\textcurrency"#, BUILTINS), // CURRENCY SIGN
    ('\u{00A5}', r#"\textyen"#, BUILTINS), // YEN SIGN
    ('\u{00A6}', r#"\textbrokenbar"#, BUILTINS), // BROKEN BAR
    ('\u{00A7}', r#"\textsection"#, BUILTINS), // SECTION SIGN
    ('\u{00A8}', r#"\textasciidieresis"#, BUILTINS), // DIAERESIS
    ('\u{00A9}', r#"\textcopyright"#, BUILTINS), // COPYRIGHT SIGN
    ('\u{00AA}', r#"\textordfeminine"#, BUILTINS), // FEMININE ORDINAL INDICATOR
    ('\u{00AB}', r#"\guillemotleft"#, FONTENC_T1), // LEFT-POINTING DOUBLE ANGLE QUOTATION MARK
    ('\u{00AC}', r#"\textlnot"#, BUILTINS), // NOT SIGN
    ('\u{00AD}', r#"\-"#, BUILTINS), // SOFT HYPHEN
    ('\u{00AE}', r#"\textregistered"#, BUILTINS), // REGISTERED SIGN
    ('\u{00AF}', r#"\textasciimacron"#, BUILTINS), // MACRON
    ('\u{00B0}', r#"\textdegree"#, BUILTINS), // DEGREE SIGN
    ('\u{00B1}', r#"\ensuremath{\pm}"#, BUILTINS), // PLUS-MINUS SIGN
    ('\u{00B2}', r#"\texttwosuperior"#, BUILTINS), // SUPERSCRIPT TWO
    ('\u{00B3}', r#"\textthreesuperior"#, BUILTINS), // SUPERSCRIPT THREE
    ('\u{00B4}', r#"\textasciiacute"#, BUILTINS), // ACUTE ACCENT
    ('\u{00B5}', r#"\textmu"#, BUILTINS), // MICRO SIGN
    ('\u{00B6}', r#"\textparagraph"#, BUILTINS), // PILCROW SIGN
    ('\u{00B7}', r#"\textperiodcentered"#, BUILTINS), // MIDDLE DOT
    ('\u{00B9}', r#"\textonesuperior"#, BUILTINS), // SUPERSCRIPT ONE
    ('\u{00BA}', r#"\textordmasculine"#, BUILTINS), // MASCULINE ORDINAL INDICATOR
    ('\u{00BB}', r#"\guillemotright"#, FONTENC_T1), // RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK
    ('\u{00BC}', r#"\textonequarter"#, BUILTINS), // VULGAR FRACTION ONE QUARTER
    ('\u{00BD}', r#"\textonehalf"#, BUILTINS), // VULGAR FRACTION ONE HALF
    ('\u{00BE}', r#"\textthreequarters"#, BUILTINS), // VULGAR FRACTION THREE QUARTERS
    ('\u{00BF}', r#"\textquestiondown"#, BUILTINS), // INVERTED QUESTION MARK
    ('\u{00C0}', r#"\`A"#, BUILTINS), // LATIN CAPITAL LETTER A WITH GRAVE
    ('\u{00C1}', r#"\'A"#, BUILTINS), // LATIN CAPITAL LETTER A WITH ACUTE
    ('\u{00C2}', r#"\^A"#, BUILTINS), // LATIN CAPITAL LETTER A WITH CIRCUMFLEX
    ('\u{00C3}', r#"\~A"#, BUILTINS), // LATIN CAPITAL LETTER A WITH TILDE
    ('\u{00C4}', r#"\"A"#, BUILTINS), // LATIN CAPITAL LETTER A WITH DIAERESIS
    ('\u{00C5}', r#"\r{A}"#, BUILTINS), // LATIN CAPITAL LETTER A WITH RING ABOVE
    ('\u{00C6}', r#"\AE"#, BUILTINS), // LATIN CAPITAL LETTER AE
    ('\u{00C7}', r#"\c{C}"#, BUILTINS), // LATIN CAPITAL LETTER C WITH CEDILLA
    ('\u{00C8}', r#"\`E"#, BUILTINS), // LATIN CAPITAL LETTER E WITH GRAVE
    ('\u{00C9}', r#"\'E"#, BUILTINS), // LATIN CAPITAL LETTER E WITH ACUTE
    ('\u{00CA}', r#"\^E"#, BUILTINS), // LATIN CAPITAL LETTER E WITH CIRCUMFLEX
    ('\u{00CB}', r#"\"E"#, BUILTINS), // LATIN CAPITAL LETTER E WITH DIAERESIS
    ('\u{00CC}', r#"\`I"#, BUILTINS), // LATIN CAPITAL LETTER I WITH GRAVE
    ('\u{00CD}', r#"\'I"#, BUILTINS), // LATIN CAPITAL LETTER I WITH ACUTE
    ('\u{00CE}', r#"\^I"#, BUILTINS), // LATIN CAPITAL LETTER I WITH CIRCUMFLEX
    ('\u{00CF}', r#"\"I"#, BUILTINS), // LATIN CAPITAL LETTER I WITH DIAERESIS
    ('\u{00D0}', r#"\DH"#, FONTENC_T1), // LATIN CAPITAL LETTER ETH
    ('\u{00D1}', r#"\~N"#, BUILTINS), // LATIN CAPITAL LETTER N WITH TILDE
    ('\u{00D2}', r#"\`O"#, BUILTINS), // LATIN CAPITAL LETTER O WITH GRAVE
    ('\u{00D3}', r#"\'O"#, BUILTINS), // LATIN CAPITAL LETTER O WITH ACUTE
    ('\u{00D4}', r#"\^O"#, BUILTINS), // LATIN CAPITAL LETTER O WITH CIRCUMFLEX
    ('\u{00D5}', r#"\~O"#, BUILTINS), // LATIN CAPITAL LETTER O WITH TILDE
    ('\u{00D6}', r#"\"O"#, BUILTINS), // LATIN CAPITAL LETTER O WITH DIAERESIS
    ('\u{00D7}', r#"\texttimes"#, BUILTINS), // MULTIPLICATION SIGN
    ('\u{00D8}', r#"\O"#, BUILTINS), // LATIN CAPITAL LETTER O WITH STROKE
    ('\u{00D9}', r#"\`U"#, BUILTINS), // LATIN CAPITAL LETTER U WITH GRAVE
    ('\u{00DA}', r#"\'U"#, BUILTINS), // LATIN CAPITAL LETTER U WITH ACUTE
    ('\u{00DB}', r#"\^U"#, BUILTINS), // LATIN CAPITAL LETTER U WITH CIRCUMFLEX
    ('\u{00DC}', r#"\"U"#, BUILTINS), // LATIN CAPITAL LETTER U WITH DIAERESIS
    ('\u{00DD}', r#"\'Y"#, BUILTINS), // LATIN CAPITAL LETTER Y WITH ACUTE
    ('\u{00DE}', r#"\TH"#, FONTENC_T1), // LATIN CAPITAL LETTER THORN
    ('\u{00DF}', r#"\ss"#, BUILTINS), // LATIN SMALL LETTER SHARP S
    ('\u{00E0}', r#"\`a"#, BUILTINS), // LATIN SMALL LETTER A WITH GRAVE
    ('\u{00E1}', r#"\'a"#, BUILTINS), // LATIN SMALL LETTER A WITH ACUTE
    ('\u{00E2}', r#"\^a"#, BUILTINS), // LATIN SMALL LETTER A WITH CIRCUMFLEX
    ('\u{00E3}', r#"\~a"#, BUILTINS), // LATIN SMALL LETTER A WITH TILDE
    ('\u{00E4}', r#"\"a"#, BUILTINS), // LATIN SMALL LETTER A WITH DIAERESIS
    ('\u{00E5}', r#"\r{a}"#, BUILTINS), // LATIN SMALL LETTER A WITH RING ABOVE
    ('\u{00E6}', r#"\ae"#, BUILTINS), // LATIN SMALL LETTER AE
    ('\u{00E7}', r#"\c{c}"#, BUILTINS), // LATIN SMALL LETTER C WITH CEDILLA
    ('\u{00E8}', r#"\`e"#, BUILTINS), // LATIN SMALL LETTER E WITH GRAVE
    ('\u{00E9}', r#"\'e"#, BUILTINS), // LATIN SMALL LETTER E WITH ACUTE
    ('\u{00EA}', r#"\^e"#, BUILTINS), // LATIN SMALL LETTER E WITH CIRCUMFLEX
    ('\u{00EB}', r#"\"e"#, BUILTINS), // LATIN SMALL LETTER E WITH DIAERESIS
    ('\u{00EC}', r#"\`i"#, BUILTINS), // LATIN SMALL LETTER I WITH GRAVE
    ('\u{00ED}', r#"\'i"#, BUILTINS), // LATIN SMALL LETTER I WITH ACUTE
    ('\u{00EE}', r#"\^i"#, BUILTINS), // LATIN SMALL LETTER I WITH CIRCUMFLEX
    ('\u{00EF}', r#"\"i"#, BUILTINS), // LATIN SMALL LETTER I WITH DIAERESIS
    ('\u{00F0}', r#"\dh"#, FONTENC_T1), // LATIN SMALL LETTER ETH
    ('\u{00F1}', r#"\~n"#, BUILTINS), // LATIN SMALL LETTER N WITH TILDE
    ('\u{00F2}', r#"\`o"#, BUILTINS), // LATIN SMALL LETTER O WITH GRAVE
    ('\u{00F3}', r#"\'o"#, BUILTINS), // LATIN SMALL LETTER O WITH ACUTE
    ('\u{00F4}', r#"\^o"#, BUILTINS), // LATIN SMALL LETTER O WITH CIRCUMFLEX
    ('\u{00F5}', r#"\~o"#, BUILTINS), // LATIN SMALL LETTER O WITH TILDE
    ('\u{00F6}', r#"\"o"#, BUILTINS), // LATIN SMALL LETTER O WITH DIAERESIS
    ('\u{00F7}', r#"\textdiv"#, BUILTINS), // DIVISION SIGN
    ('\u{00F8}', r#"\o"#, BUILTINS), // LATIN SMALL LETTER O WITH STROKE
    ('\u{00F9}', r#"\`u"#, BUILTINS), // LATIN SMALL LETTER U WITH GRAVE
    ('\u{00FA}', r#"\'u"#, BUILTINS), // LATIN SMALL LETTER U WITH ACUTE
    ('\u{00FB}', r#"\^u"#, BUILTINS), // LATIN SMALL LETTER U WITH CIRCUMFLEX
    ('\u{00FC}', r#"\"u"#, BUILTINS), // LATIN SMALL LETTER U WITH DIAERESIS
    ('\u{00FD}', r#"\'y"#, BUILTINS), // LATIN SMALL LETTER Y WITH ACUTE
    ('\u{00FE}', r#"\th"#, FONTENC_T1), // LATIN SMALL LETTER THORN
    ('\u{00FF}', r#"\"y"#, BUILTINS), // LATIN SMALL LETTER Y WITH DIAERESIS
    ('\u{0100}', r#"\={A}"#, BUILTINS), // LATIN CAPITAL LETTER A WITH MACRON
    ('\u{0101}', r#"\={a}"#, BUILTINS), // LATIN SMALL LETTER A WITH MACRON
    ('\u{0102}', r#"\u{A}"#, BUILTINS), // LATIN CAPITAL LETTER A WITH BREVE
    ('\u{0103}', r#"\u{a}"#, BUILTINS), // LATIN SMALL LETTER A WITH BREVE
    ('\u{0104}', r#"\k{A}"#, FONTENC_T1), // LATIN CAPITAL LETTER A WITH OGONEK
    ('\u{0105}', r#"\k{a}"#, FONTENC_T1), // LATIN SMALL LETTER A WITH OGONEK
    ('\u{0106}', r#"\'C"#, BUILTINS), // LATIN CAPITAL LETTER C WITH ACUTE
    ('\u{0107}', r#"\'c"#, BUILTINS), // LATIN SMALL LETTER C WITH ACUTE
    ('\u{0108}', r#"\^{C}"#, BUILTINS), // LATIN CAPITAL LETTER C WITH CIRCUMFLEX
    ('\u{0109}', r#"\^{c}"#, BUILTINS), // LATIN SMALL LETTER C WITH CIRCUMFLEX
    ('\u{010A}', r#"\.{C}"#, BUILTINS), // LATIN CAPITAL LETTER C WITH DOT ABOVE
    ('\u{010B}', r#"\.{c}"#, BUILTINS), // LATIN SMALL LETTER C WITH DOT ABOVE
    ('\u{010C}', r#"\v{C}"#, BUILTINS), // LATIN CAPITAL LETTER C WITH CARON
    ('\u{010D}', r#"\v{c}"#, BUILTINS), // LATIN SMALL LETTER C WITH CARON
    ('\u{010E}', r#"\v{D}"#, BUILTINS), // LATIN CAPITAL LETTER D WITH CARON
    ('\u{010F}', r#"\v{d}"#, BUILTINS), // LATIN SMALL LETTER D WITH CARON
    ('\u{0110}', r#"\DJ"#, FONTENC_T1), // LATIN CAPITAL LETTER D WITH STROKE
    ('\u{0111}', r#"\dj"#, FONTENC_T1), // LATIN SMALL LETTER D WITH STROKE
    ('\u{0112}', r#"\={E}"#, BUILTINS), // LATIN CAPITAL LETTER E WITH MACRON
    ('\u{0113}', r#"\={e}"#, BUILTINS), // LATIN SMALL LETTER E WITH MACRON
    ('\u{0114}', r#"\u{E}"#, BUILTINS), // LATIN CAPITAL LETTER E WITH BREVE
    ('\u{0115}', r#"\u{e}"#, BUILTINS), // LATIN SMALL LETTER E WITH BREVE
    ('\u{0116}', r#"\.{E}"#, BUILTINS), // LATIN CAPITAL LETTER E WITH DOT ABOVE
    ('\u{0117}', r#"\.{e}"#, BUILTINS), // LATIN SMALL LETTER E WITH DOT ABOVE
    ('\u{0118}', r#"\k{E}"#, FONTENC_T1), // LATIN CAPITAL LETTER E WITH OGONEK
    ('\u{0119}', r#"\k{e}"#, FONTENC_T1), // LATIN SMALL LETTER E WITH OGONEK
    ('\u{011A}', r#"\v{E}"#, BUILTINS), // LATIN CAPITAL LETTER E WITH CARON
    ('\u{011B}', r#"\v{e}"#, BUILTINS), // LATIN SMALL LETTER E WITH CARON
    ('\u{011C}', r#"\^{G}"#, BUILTINS), // LATIN CAPITAL LETTER G WITH CIRCUMFLEX
    ('\u{011D}', r#"\^{g}"#, BUILTINS), // LATIN SMALL LETTER G WITH CIRCUMFLEX
    ('\u{011E}', r#"\u{G}"#, BUILTINS), // LATIN CAPITAL LETTER G WITH BREVE
    ('\u{011F}', r#"\u{g}"#, BUILTINS), // LATIN SMALL LETTER G WITH BREVE
    ('\u{0120}', r#"\.{G}"#, BUILTINS), // LATIN CAPITAL LETTER G WITH DOT ABOVE
    ('\u{0121}', r#"\.{g}"#, BUILTINS), // LATIN SMALL LETTER G WITH DOT ABOVE
    ('\u{0122}', r#"\c{G}"#, BUILTINS), // LATIN CAPITAL LETTER G WITH CEDILLA
    ('\u{0123}', r#"\c{g}"#, BUILTINS), // LATIN SMALL LETTER G WITH CEDILLA
    ('\u{0124}', r#"\^{H}"#, BUILTINS), // LATIN CAPITAL LETTER H WITH CIRCUMFLEX
    ('\u{0125}', r#"\^{h}"#, BUILTINS), // LATIN SMALL LETTER H WITH CIRCUMFLEX
    ('\u{0126}', r#"\={H}"#, BUILTINS), // LATIN CAPITAL LETTER H WITH STROKE
    ('\u{0127}', r#"\={h}"#, BUILTINS), // LATIN SMALL LETTER H WITH STROKE
    ('\u{0128}', r#"\~{I}"#, BUILTINS), // LATIN CAPITAL LETTER I WITH TILDE
    ('\u{0129}', r#"\~{i}"#, BUILTINS), // LATIN SMALL LETTER I WITH TILDE
    ('\u{012A}', r#"\={I}"#, BUILTINS), // LATIN CAPITAL LETTER I WITH MACRON
    ('\u{012B}', r#"\={i}"#, BUILTINS), // LATIN SMALL LETTER I WITH MACRON
    ('\u{012C}', r#"\u{I}"#, BUILTINS), // LATIN CAPITAL LETTER I WITH BREVE
    ('\u{012D}', r#"\u{i}"#, BUILTINS), // LATIN SMALL LETTER I WITH BREVE
    ('\u{012E}', r#"\k{I}"#, FONTENC_T1), // LATIN CAPITAL LETTER I WITH OGONEK
    ('\u{012F}', r#"\k{i}"#, FONTENC_T1), // LATIN SMALL LETTER I WITH OGONEK
    ('\u{0130}', r#"\.I"#, BUILTINS), // LATIN CAPITAL LETTER I WITH DOT ABOVE
    ('\u{0131}', r#"\i"#, BUILTINS), // LATIN SMALL LETTER DOTLESS I
    ('\u{0132}', r#"\IJ"#, BUILTINS), // LATIN CAPITAL LIGATURE IJ
    ('\u{0133}', r#"\ij"#, BUILTINS), // LATIN SMALL LIGATURE IJ
    ('\u{0134}', r#"\^{J}"#, BUILTINS), // LATIN CAPITAL LETTER J WITH CIRCUMFLEX
    ('\u{0135}', r#"\^{j}"#, BUILTINS), // LATIN SMALL LETTER J WITH CIRCUMFLEX
    ('\u{0136}', r#"\c{K}"#, BUILTINS), // LATIN CAPITAL LETTER K WITH CEDILLA
    ('\u{0137}', r#"\c{k}"#, BUILTINS), // LATIN SMALL LETTER K WITH CEDILLA
    ('\u{0138}', r#"\textsc{k}"#, BUILTINS), // LATIN SMALL LETTER KRA
    ('\u{0139}', r#"\'L"#, BUILTINS), // LATIN CAPITAL LETTER L WITH ACUTE
    ('\u{013A}', r#"\'l"#, BUILTINS), // LATIN SMALL LETTER L WITH ACUTE
    ('\u{013B}', r#"\c{L}"#, BUILTINS), // LATIN CAPITAL LETTER L WITH CEDILLA
    ('\u{013C}', r#"\c{l}"#, BUILTINS), // LATIN SMALL LETTER L WITH CEDILLA
    ('\u{013D}', r#"\v{L}"#, BUILTINS), // LATIN CAPITAL LETTER L WITH CARON
    ('\u{013E}', r#"\v{l}"#, BUILTINS), // LATIN SMALL LETTER L WITH CARON
    ('\u{013F}', r#"\.{L}"#, BUILTINS), // LATIN CAPITAL LETTER L WITH MIDDLE DOT
    ('\u{0140}', r#"\.{l}"#, BUILTINS), // LATIN SMALL LETTER L WITH MIDDLE DOT
    ('\u{0141}', r#"\L"#, BUILTINS), // LATIN CAPITAL LETTER L WITH STROKE
    ('\u{0142}', r#"\l"#, BUILTINS), // LATIN SMALL LETTER L WITH STROKE
    ('\u{0143}', r#"\'N"#, BUILTINS), // LATIN CAPITAL LETTER N WITH ACUTE
    ('\u{0144}', r#"\'n"#, BUILTINS), // LATIN SMALL LETTER N WITH ACUTE
    ('\u{0145}', r#"\c{N}"#, BUILTINS), // LATIN CAPITAL LETTER N WITH CEDILLA
    ('\u{0146}', r#"\c{n}"#, BUILTINS), // LATIN SMALL LETTER N WITH CEDILLA
    ('\u{0147}', r#"\v{N}"#, BUILTINS), // LATIN CAPITAL LETTER N WITH CARON
    ('\u{0148}', r#"\v{n}"#, BUILTINS), // LATIN SMALL LETTER N WITH CARON
    ('\u{0149}', r#"\textquoteright n"#, BUILTINS), // LATIN SMALL LETTER N PRECEDED BY APOSTROPHE
    ('\u{014A}', r#"\NG"#, FONTENC_T1), // LATIN CAPITAL LETTER ENG
    ('\u{014B}', r#"\ng"#, FONTENC_T1), // LATIN SMALL LETTER ENG
    ('\u{014C}', r#"\={O}"#, BUILTINS), // LATIN CAPITAL LETTER O WITH MACRON
    ('\u{014D}', r#"\={o}"#, BUILTINS), // LATIN SMALL LETTER O WITH MACRON
    ('\u{014E}', r#"\u{O}"#, BUILTINS), // LATIN CAPITAL LETTER O WITH BREVE
    ('\u{014F}', r#"\u{o}"#, BUILTINS), // LATIN SMALL LETTER O WITH BREVE
    ('\u{0150}', r#"\H{O}"#, BUILTINS), // LATIN CAPITAL LETTER O WITH DOUBLE ACUTE
    ('\u{0151}', r#"\H{o}"#, BUILTINS), // LATIN SMALL LETTER O WITH DOUBLE ACUTE
    ('\u{0152}', r#"\OE"#, BUILTINS), // LATIN CAPITAL LIGATURE OE
    ('\u{0153}', r#"\oe"#, BUILTINS), // LATIN SMALL LIGATURE OE
    ('\u{0154}', r#"\'R"#, BUILTINS), // LATIN CAPITAL LETTER R WITH ACUTE
    ('\u{0155}', r#"\'r"#, BUILTINS), // LATIN SMALL LETTER R WITH ACUTE
    ('\u{0156}', r#"\c{R}"#, BUILTINS), // LATIN CAPITAL LETTER R WITH CEDILLA
    ('\u{0157}', r#"\c{r}"#, BUILTINS), // LATIN SMALL LETTER R WITH CEDILLA
    ('\u{0158}', r#"\v{R}"#, BUILTINS), // LATIN CAPITAL LETTER R WITH CARON
    ('\u{0159}', r#"\v{r}"#, BUILTINS), // LATIN SMALL LETTER R WITH CARON
    ('\u{015A}', r#"\'S"#, BUILTINS), // LATIN CAPITAL LETTER S WITH ACUTE
    ('\u{015B}', r#"\'s"#, BUILTINS), // LATIN SMALL LETTER S WITH ACUTE
    ('\u{015C}', r#"\^{S}"#, BUILTINS), // LATIN CAPITAL LETTER S WITH CIRCUMFLEX
    ('\u{015D}', r#"\^{s}"#, BUILTINS), // LATIN SMALL LETTER S WITH CIRCUMFLEX
    ('\u{015E}', r#"\c{S}"#, BUILTINS), // LATIN CAPITAL LETTER S WITH CEDILLA
    ('\u{015F}', r#"\c{s}"#, BUILTINS), // LATIN SMALL LETTER S WITH CEDILLA
    ('\u{0160}', r#"\v{S}"#, BUILTINS), // LATIN CAPITAL LETTER S WITH CARON
    ('\u{0161}', r#"\v{s}"#, BUILTINS), // LATIN SMALL LETTER S WITH CARON
    ('\u{0162}', r#"\c{T}"#, BUILTINS), // LATIN CAPITAL LETTER T WITH CEDILLA
    ('\u{0163}', r#"\c{t}"#, BUILTINS), // LATIN SMALL LETTER T WITH CEDILLA
    ('\u{0164}', r#"\v{T}"#, BUILTINS), // LATIN CAPITAL LETTER T WITH CARON
    ('\u{0165}', r#"\v{t}"#, BUILTINS), // LATIN SMALL LETTER T WITH CARON
    ('\u{0166}', r#"\={T}"#, BUILTINS), // LATIN CAPITAL LETTER T WITH STROKE
    ('\u{0167}', r#"\={t}"#, BUILTINS), // LATIN SMALL LETTER T WITH STROKE
    ('\u{0168}', r#"\~{U}"#, BUILTINS), // LATIN CAPITAL LETTER U WITH TILDE
    ('\u{0169}', r#"\~{u}"#, BUILTINS), // LATIN SMALL LETTER U WITH TILDE
    ('\u{016A}', r#"\={U}"#, BUILTINS), // LATIN CAPITAL LETTER U WITH MACRON
    ('\u{016B}', r#"\={u}"#, BUILTINS), // LATIN SMALL LETTER U WITH MACRON
    ('\u{016C}', r#"\u{U}"#, BUILTINS), // LATIN CAPITAL LETTER U WITH BREVE
    ('\u{016D}', r#"\u{u}"#, BUILTINS), // LATIN SMALL LETTER U WITH BREVE
    ('\u{016E}', r#"\r{U}"#, BUILTINS), // LATIN CAPITAL LETTER U WITH RING ABOVE
    ('\u{016F}', r#"\r{u}"#, BUILTINS), // LATIN SMALL LETTER U WITH RING ABOVE
    ('\u{0170}', r#"\'{U}"#, BUILTINS), // LATIN CAPITAL LETTER U WITH DOUBLE ACUTE
    ('\u{0171}', r#"\'{u}"#, BUILTINS), // LATIN SMALL LETTER U WITH DOUBLE ACUTE
    ('\u{0172}', r#"\k{U}"#, FONTENC_T1), // LATIN CAPITAL LETTER U WITH OGONEK
    ('\u{0173}', r#"\k{u}"#, FONTENC_T1), // LATIN SMALL LETTER U WITH OGONEK
    ('\u{0174}', r#"\^{W}"#, BUILTINS), // LATIN CAPITAL LETTER W WITH CIRCUMFLEX
    ('\u{0175}', r#"\^{w}"#, BUILTINS), // LATIN SMALL LETTER W WITH CIRCUMFLEX
    ('\u{0176}', r#"\^{Y}"#, BUILTINS), // LATIN CAPITAL LETTER Y WITH CIRCUMFLEX
    ('\u{0177}', r#"\^{y}"#, BUILTINS), // LATIN SMALL LETTER Y WITH CIRCUMFLEX
    ('\u{0178}', r#"\"Y"#, BUILTINS), // LATIN CAPITAL LETTER Y WITH DIAERESIS
    ('\u{0179}', r#"\'Z"#, BUILTINS), // LATIN CAPITAL LETTER Z WITH ACUTE
    ('\u{017A}', r#"\'z"#, BUILTINS), // LATIN SMALL LETTER Z WITH ACUTE
    ('\u{017B}', r#"\.Z"#, BUILTINS), // LATIN CAPITAL LETTER Z WITH DOT ABOVE
    ('\u{017C}', r#"\.z"#, BUILTINS), // LATIN SMALL LETTER Z WITH DOT ABOVE
    ('\u{017D}', r#"\v{Z}"#, BUILTINS), // LATIN CAPITAL LETTER Z WITH CARON
    ('\u{017E}', r#"\v{z}"#, BUILTINS), // LATIN SMALL LETTER Z WITH CARON
    ('\u{0192}', r#"\textflorin"#, BUILTINS), // LATIN SMALL LETTER F WITH HOOK
    ('\u{0195}', r#"\texthvlig"#, TIPA), // LATIN SMALL LETTER HV
    ('\u{019E}', r#"\textnrleg"#, TIPX), // LATIN SMALL LETTER N WITH LONG RIGHT LEG
    ('\u{01E7}', r#"\v{g}"#, BUILTINS), // LATIN SMALL LETTER G WITH CARON
    ('\u{01F5}', r#"\'{g}"#, BUILTINS), // LATIN SMALL LETTER G WITH ACUTE
    ('\u{0228}', r#"\c{E}"#, BUILTINS), // LATIN CAPITAL LETTER E WITH CEDILLA
    ('\u{0229}', r#"\c{e}"#, BUILTINS), // LATIN SMALL LETTER E WITH CEDILLA
    ('\u{0259}', r#"\textschwa"#, TIPA), // LATIN SMALL LETTER SCHWA
    ('\u{025B}', r#"\ensuremath{\varepsilon}"#, BUILTINS), // LATIN SMALL LETTER OPEN E
    ('\u{0278}', r#"\textphi"#, TIPA), // LATIN SMALL LETTER PHI
    ('\u{0294}', r#"\textglotstop"#, TIPA), // LATIN LETTER GLOTTAL STOP
    ('\u{029E}', r#"\textturnk"#, TIPA), // LATIN SMALL LETTER TURNED K
    ('\u{02B7}', r#"\textsuperscript{w}"#, BUILTINS), // MODIFIER LETTER SMALL W
    ('\u{02BC}', r#"'"#, BUILTINS), // MODIFIER LETTER APOSTROPHE
    ('\u{02C6}', r#"\textasciicircum"#, BUILTINS), // MODIFIER LETTER CIRCUMFLEX ACCENT
    ('\u{02C7}', r#"\textasciicaron"#, BUILTINS), // CARON
    ('\u{02D8}', r#"\textasciibreve"#, BUILTINS), // BREVE
    ('\u{02D9}', r#"\textperiodcentered"#, BUILTINS), // DOT ABOVE
    ('\u{02DA}', r#"\r{}"#, BUILTINS), // RING ABOVE
    ('\u{02DB}', r#"\k{}"#, FONTENC_T1), // OGONEK
    ('\u{02DC}', r#"\textasciitilde"#, BUILTINS), // SMALL TILDE
    ('\u{02DD}', r#"\textacutedbl"#, BUILTINS), // DOUBLE ACUTE ACCENT
    ('\u{0386}', r#"\'{}A"#, BUILTINS), // GREEK CAPITAL LETTER ALPHA WITH TONOS
    ('\u{0388}', r#"\'{}E"#, BUILTINS), // GREEK CAPITAL LETTER EPSILON WITH TONOS
    ('\u{0389}', r#"\'{}H"#, BUILTINS), // GREEK CAPITAL LETTER ETA WITH TONOS
    ('\u{038A}', r#"\'{}I"#, BUILTINS), // GREEK CAPITAL LETTER IOTA WITH TONOS
    ('\u{038C}', r#"\'{}O"#, BUILTINS), // GREEK CAPITAL LETTER OMICRON WITH TONOS
    ('\u{038E}', r#"\'{}Y"#, BUILTINS), // GREEK CAPITAL LETTER UPSILON WITH TONOS
    ('\u{038F}', r#"\'{}\ensuremath{\Omega}"#, BUILTINS), // GREEK CAPITAL LETTER OMEGA WITH TONOS
    ('\u{0390}', r#"\ensuremath{\acute{\ddot{\iota}}}"#, BUILTINS), // GREEK SMALL LETTER IOTA WITH DIALYTIKA AND TONOS
    ('\u{0391}', r#"A"#, BUILTINS), // GREEK CAPITAL LETTER ALPHA
    ('\u{0392}', r#"B"#, BUILTINS), // GREEK CAPITAL LETTER BETA
    ('\u{0393}', r#"\ensuremath{\Gamma}"#, BUILTINS), // GREEK CAPITAL LETTER GAMMA
    ('\u{0394}', r#"\ensuremath{\Delta}"#, BUILTINS), // GREEK CAPITAL LETTER DELTA
    ('\u{0395}', r#"E"#, BUILTINS), // GREEK CAPITAL LETTER EPSILON
    ('\u{0396}', r#"Z"#, BUILTINS), // GREEK CAPITAL LETTER ZETA
    ('\u{0397}', r#"H"#, BUILTINS), // GREEK CAPITAL LETTER ETA
    ('\u{0398}', r#"\ensuremath{\Theta}"#, BUILTINS), // GREEK CAPITAL LETTER THETA
    ('\u{0399}', r#"I"#, BUILTINS), // GREEK CAPITAL LETTER IOTA
    ('\u{039A}', r#"K"#, BUILTINS), // GREEK CAPITAL LETTER KAPPA
    ('\u{039B}', r#"\ensuremath{\Lambda}"#, BUILTINS), // GREEK CAPITAL LETTER LAMDA
    ('\u{039C}', r#"M"#, BUILTINS), // GREEK CAPITAL LETTER MU
    ('\u{039D}', r#"N"#, BUILTINS), // GREEK CAPITAL LETTER NU
    ('\u{039E}', r#"\ensuremath{\Xi}"#, BUILTINS), // GREEK CAPITAL LETTER XI
    ('\u{039F}', r#"O"#, BUILTINS), // GREEK CAPITAL LETTER OMICRON
    ('\u{03A0}', r#"\ensuremath{\Pi}"#, BUILTINS), // GREEK CAPITAL LETTER PI
    ('\u{03A1}', r#"P"#, BUILTINS), // GREEK CAPITAL LETTER RHO
    ('\u{03A3}', r#"\ensuremath{\Sigma}"#, BUILTINS), // GREEK CAPITAL LETTER SIGMA
    ('\u{03A4}', r#"T"#, BUILTINS), // GREEK CAPITAL LETTER TAU
    ('\u{03A5}', r#"\ensuremath{\Upsilon}"#, BUILTINS), // GREEK CAPITAL LETTER UPSILON
    ('\u{03A6}', r#"\ensuremath{\Phi}"#, BUILTINS), // GREEK CAPITAL LETTER PHI
    ('\u{03A7}', r#"X"#, BUILTINS), // GREEK CAPITAL LETTER CHI
    ('\u{03A8}', r#"\ensuremath{\Psi}"#, BUILTINS), // GREEK CAPITAL LETTER PSI
    ('\u{03A9}', r#"\ensuremath{\Omega}"#, BUILTINS), // GREEK CAPITAL LETTER OMEGA
    ('\u{03AA}', r#"\ensuremath{\ddot{I}}"#, BUILTINS), // GREEK CAPITAL LETTER IOTA WITH DIALYTIKA
    ('\u{03AB}', r#"\ensuremath{\ddot{Y}}"#, BUILTINS), // GREEK CAPITAL LETTER UPSILON WITH DIALYTIKA
    ('\u{03AC}', r#"\ensuremath{\acute\alpha}"#, BUILTINS), // GREEK SMALL LETTER ALPHA WITH TONOS
    ('\u{03AD}', r#"\ensuremath{\acute\epsilon}"#, BUILTINS), // GREEK SMALL LETTER EPSILON WITH TONOS
    ('\u{03AE}', r#"\ensuremath{\acute\eta}"#, BUILTINS), // GREEK SMALL LETTER ETA WITH TONOS
    ('\u{03AF}', r#"\ensuremath{\acute\iota}"#, BUILTINS), // GREEK SMALL LETTER IOTA WITH TONOS
    ('\u{03B0}', r#"\ensuremath{\acute{\ddot{\upsilon}}}"#, BUILTINS), // GREEK SMALL LETTER UPSILON WITH DIALYTIKA AND TONOS
    ('\u{03B1}', r#"\ensuremath{\alpha}"#, BUILTINS), // GREEK SMALL LETTER ALPHA
    ('\u{03B2}', r#"\ensuremath{\beta}"#, BUILTINS), // GREEK SMALL LETTER BETA
    ('\u{03B3}', r#"\ensuremath{\gamma}"#, BUILTINS), // GREEK SMALL LETTER GAMMA
    ('\u{03B4}', r#"\ensuremath{\delta}"#, BUILTINS), // GREEK SMALL LETTER DELTA
    ('\u{03B5}', r#"\ensuremath{\varepsilon}"#, BUILTINS), // GREEK SMALL LETTER EPSILON
    ('\u{03B6}', r#"\ensuremath{\zeta}"#, BUILTINS), // GREEK SMALL LETTER ZETA
    ('\u{03B7}', r#"\ensuremath{\eta}"#, BUILTINS), // GREEK SMALL LETTER ETA
    ('\u{03B8}', r#"\ensuremath{\theta}"#, BUILTINS), // GREEK SMALL LETTER THETA
    ('\u{03B9}', r#"\ensuremath{\iota}"#, BUILTINS), // GREEK SMALL LETTER IOTA
    ('\u{03BA}', r#"\ensuremath{\kappa}"#, BUILTINS), // GREEK SMALL LETTER KAPPA
    ('\u{03BB}', r#"\ensuremath{\lambda}"#, BUILTINS), // GREEK SMALL LETTER LAMDA
    ('\u{03BC}', r#"\ensuremath{\mu}"#, BUILTINS), // GREEK SMALL LETTER MU
    ('\u{03BD}', r#"\ensuremath{\nu}"#, BUILTINS), // GREEK SMALL LETTER NU
    ('\u{03BE}', r#"\ensuremath{\xi}"#, BUILTINS), // GREEK SMALL LETTER XI
    ('\u{03BF}', r#"o"#, BUILTINS), // GREEK SMALL LETTER OMICRON
    ('\u{03C0}', r#"\ensuremath{\pi}"#, BUILTINS), // GREEK SMALL LETTER PI
    ('\u{03C1}', r#"\ensuremath{\rho}"#, BUILTINS), // GREEK SMALL LETTER RHO
    ('\u{03C2}', r#"\ensuremath{\varsigma}"#, BUILTINS), // GREEK SMALL LETTER FINAL SIGMA
    ('\u{03C3}', r#"\ensuremath{\sigma}"#, BUILTINS), // GREEK SMALL LETTER SIGMA
    ('\u{03C4}', r#"\ensuremath{\tau}"#, BUILTINS), // GREEK SMALL LETTER TAU
    ('\u{03C5}', r#"\ensuremath{\upsilon}"#, BUILTINS), // GREEK SMALL LETTER UPSILON
    ('\u{03C6}', r#"\ensuremath{\varphi}"#, BUILTINS), // GREEK SMALL LETTER PHI
    ('\u{03C7}', r#"\ensuremath{\chi}"#, BUILTINS), // GREEK SMALL LETTER CHI
    ('\u{03C8}', r#"\ensuremath{\psi}"#, BUILTINS), // GREEK SMALL LETTER PSI
    ('\u{03C9}', r#"\ensuremath{\omega}"#, BUILTINS), // GREEK SMALL LETTER OMEGA
    ('\u{03CA}', r#"\ensuremath{\ddot\iota}"#, BUILTINS), // GREEK SMALL LETTER IOTA WITH DIALYTIKA
    ('\u{03CB}', r#"\ensuremath{\ddot{\upsilon}}"#, BUILTINS), // GREEK SMALL LETTER UPSILON WITH DIALYTIKA
    ('\u{03CC}', r#"\'{o}"#, BUILTINS), // GREEK SMALL LETTER OMICRON WITH TONOS
    ('\u{03CD}', r#"\ensuremath{\acute\upsilon}"#, BUILTINS), // GREEK SMALL LETTER UPSILON WITH TONOS
    ('\u{03CE}', r#"\ensuremath{\acute\omega}"#, BUILTINS), // GREEK SMALL LETTER OMEGA WITH TONOS
    ('\u{03D1}', r#"\ensuremath{\vartheta}"#, BUILTINS), // GREEK THETA SYMBOL
    ('\u{03D2}', r#"\ensuremath{\Upsilon}"#, BUILTINS), // GREEK UPSILON WITH HOOK SYMBOL
    ('\u{03D5}', r#"\ensuremath{\phi}"#, BUILTINS), // GREEK PHI SYMBOL
    ('\u{03D6}', r#"\ensuremath{\varpi}"#, BUILTINS), // GREEK PI SYMBOL
    ('\u{03F0}', r#"\ensuremath{\varkappa}"#, AMSSYMB), // GREEK KAPPA SYMBOL
    ('\u{03F1}', r#"\ensuremath{\varrho}"#, BUILTINS), // GREEK RHO SYMBOL
    ('\u{03F5}', r#"\ensuremath{\epsilon}"#, BUILTINS), // GREEK LUNATE EPSILON SYMBOL
    ('\u{03F6}', r#"\ensuremath{\backepsilon}"#, AMSSYMB), // GREEK REVERSED LUNATE EPSILON SYMBOL
    ('\u{0400}', r#"{\fontencoding{T2A}\selectfont\`\CYRE}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER IE WITH GRAVE
    ('\u{0401}', r#"{\fontencoding{T2A}\selectfont\CYRYO}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER IO
    ('\u{0402}', r#"{\fontencoding{T2A}\selectfont\CYRDJE}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER DJE
    ('\u{0403}', r#"{\fontencoding{T2A}\selectfont\`\CYRG}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER GJE
    ('\u{0404}', r#"{\fontencoding{T2A}\selectfont\CYRIE}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER UKRAINIAN IE
    ('\u{0405}', r#"{\fontencoding{T2A}\selectfont\CYRDZE}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER DZE
    ('\u{0406}', r#"{\fontencoding{T2A}\selectfont\CYRII}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER BYELORUSSIAN-UKRAINIAN I
    ('\u{0407}', r#"{\fontencoding{T2A}\selectfont\CYRYI}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER YI
    ('\u{0408}', r#"{\fontencoding{T2A}\selectfont\CYRJE}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER JE
    ('\u{0409}', r#"{\fontencoding{T2A}\selectfont\CYRLJE}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER LJE
    ('\u{040A}', r#"{\fontencoding{T2A}\selectfont\CYRNJE}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER NJE
    ('\u{040B}', r#"{\fontencoding{T2A}\selectfont\CYRTSHE}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER TSHE
    ('\u{040C}', r#"{\fontencoding{T2A}\selectfont\`\CYRK}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER KJE
    ('\u{040D}', r#"{\fontencoding{T2A}\selectfont\`\CYRI}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER I WITH GRAVE
    ('\u{040E}', r#"{\fontencoding{T2A}\selectfont\CYRUSHRT}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SHORT U
    ('\u{040F}', r#"{\fontencoding{T2A}\selectfont\CYRDZHE}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER DZHE
    ('\u{0410}', r#"{\fontencoding{T2A}\selectfont\CYRA}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER A
    ('\u{0411}', r#"{\fontencoding{T2A}\selectfont\CYRB}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER BE
    ('\u{0412}', r#"{\fontencoding{T2A}\selectfont\CYRV}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER VE
    ('\u{0413}', r#"{\fontencoding{T2A}\selectfont\CYRG}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER GHE
    ('\u{0414}', r#"{\fontencoding{T2A}\selectfont\CYRD}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER DE
    ('\u{0415}', r#"{\fontencoding{T2A}\selectfont\CYRE}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER IE
    ('\u{0416}', r#"{\fontencoding{T2A}\selectfont\CYRZH}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ZHE
    ('\u{0417}', r#"{\fontencoding{T2A}\selectfont\CYRZ}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ZE
    ('\u{0418}', r#"{\fontencoding{T2A}\selectfont\CYRI}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER I
    ('\u{0419}', r#"{\fontencoding{T2A}\selectfont\CYRISHRT}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SHORT I
    ('\u{041A}', r#"{\fontencoding{T2A}\selectfont\CYRK}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER KA
    ('\u{041B}', r#"{\fontencoding{T2A}\selectfont\CYRL}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER EL
    ('\u{041C}', r#"{\fontencoding{T2A}\selectfont\CYRM}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER EM
    ('\u{041D}', r#"{\fontencoding{T2A}\selectfont\CYRN}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER EN
    ('\u{041E}', r#"{\fontencoding{T2A}\selectfont\CYRO}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER O
    ('\u{041F}', r#"{\fontencoding{T2A}\selectfont\CYRP}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER PE
    ('\u{0420}', r#"{\fontencoding{T2A}\selectfont\CYRR}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ER
    ('\u{0421}', r#"{\fontencoding{T2A}\selectfont\CYRS}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ES
    ('\u{0422}', r#"{\fontencoding{T2A}\selectfont\CYRT}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER TE
    ('\u{0423}', r#"{\fontencoding{T2A}\selectfont\CYRU}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER U
    ('\u{0424}', r#"{\fontencoding{T2A}\selectfont\CYRF}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER EF
    ('\u{0425}', r#"{\fontencoding{T2A}\selectfont\CYRH}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER HA
    ('\u{0426}', r#"{\fontencoding{T2A}\selectfont\CYRC}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER TSE
    ('\u{0427}', r#"{\fontencoding{T2A}\selectfont\CYRCH}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER CHE
    ('\u{0428}', r#"{\fontencoding{T2A}\selectfont\CYRSH}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SHA
    ('\u{0429}', r#"{\fontencoding{T2A}\selectfont\CYRSHCH}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SHCHA
    ('\u{042A}', r#"{\fontencoding{T2A}\selectfont\CYRHRDSN}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER HARD SIGN
    ('\u{042B}', r#"{\fontencoding{T2A}\selectfont\CYRERY}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER YERU
    ('\u{042C}', r#"{\fontencoding{T2A}\selectfont\CYRSFTSN}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SOFT SIGN
    ('\u{042D}', r#"{\fontencoding{T2A}\selectfont\CYREREV}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER E
    ('\u{042E}', r#"{\fontencoding{T2A}\selectfont\CYRYU}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER YU
    ('\u{042F}', r#"{\fontencoding{T2A}\selectfont\CYRYA}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER YA
    ('\u{0430}', r#"{\fontencoding{T2A}\selectfont\cyra}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER A
    ('\u{0431}', r#"{\fontencoding{T2A}\selectfont\cyrb}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER BE
    ('\u{0432}', r#"{\fontencoding{T2A}\selectfont\cyrv}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER VE
    ('\u{0433}', r#"{\fontencoding{T2A}\selectfont\cyrg}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER GHE
    ('\u{0434}', r#"{\fontencoding{T2A}\selectfont\cyrd}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER DE
    ('\u{0435}', r#"{\fontencoding{T2A}\selectfont\cyre}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER IE
    ('\u{0436}', r#"{\fontencoding{T2A}\selectfont\cyrzh}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER ZHE
    ('\u{0437}', r#"{\fontencoding{T2A}\selectfont\cyrz}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER ZE
    ('\u{0438}', r#"{\fontencoding{T2A}\selectfont\cyri}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER I
    ('\u{0439}', r#"{\fontencoding{T2A}\selectfont\cyrishrt}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER SHORT I
    ('\u{043A}', r#"{\fontencoding{T2A}\selectfont\cyrk}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER KA
    ('\u{043B}', r#"{\fontencoding{T2A}\selectfont\cyrl}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER EL
    ('\u{043C}', r#"{\fontencoding{T2A}\selectfont\cyrm}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER EM
    ('\u{043D}', r#"{\fontencoding{T2A}\selectfont\cyrn}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER EN
    ('\u{043E}', r#"{\fontencoding{T2A}\selectfont\cyro}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER O
    ('\u{043F}', r#"{\fontencoding{T2A}\selectfont\cyrp}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER PE
    ('\u{0440}', r#"{\fontencoding{T2A}\selectfont\cyrr}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER ER
    ('\u{0441}', r#"{\fontencoding{T2A}\selectfont\cyrs}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER ES
    ('\u{0442}', r#"{\fontencoding{T2A}\selectfont\cyrt}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER TE
    ('\u{0443}', r#"{\fontencoding{T2A}\selectfont\cyru}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER U
    ('\u{0444}', r#"{\fontencoding{T2A}\selectfont\cyrf}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER EF
    ('\u{0445}', r#"{\fontencoding{T2A}\selectfont\cyrh}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER HA
    ('\u{0446}', r#"{\fontencoding{T2A}\selectfont\cyrc}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER TSE
    ('\u{0447}', r#"{\fontencoding{T2A}\selectfont\cyrch}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER CHE
    ('\u{0448}', r#"{\fontencoding{T2A}\selectfont\cyrsh}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER SHA
    ('\u{0449}', r#"{\fontencoding{T2A}\selectfont\cyrshch}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER SHCHA
    ('\u{044A}', r#"{\fontencoding{T2A}\selectfont\cyrhrdsn}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER HARD SIGN
    ('\u{044B}', r#"{\fontencoding{T2A}\selectfont\cyrery}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER YERU
    ('\u{044C}', r#"{\fontencoding{T2A}\selectfont\cyrsftsn}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER SOFT SIGN
    ('\u{044D}', r#"{\fontencoding{T2A}\selectfont\cyrerev}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER E
    ('\u{044E}', r#"{\fontencoding{T2A}\selectfont\cyryu}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER YU
    ('\u{044F}', r#"{\fontencoding{T2A}\selectfont\cyrya}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER YA
    ('\u{0450}', r#"{\fontencoding{T2A}\selectfont\`\cyre}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER IE WITH GRAVE
    ('\u{0451}', r#"{\fontencoding{T2A}\selectfont\cyryo}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER IO
    ('\u{0452}', r#"{\fontencoding{T2A}\selectfont\cyrdje}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER DJE
    ('\u{0453}', r#"{\fontencoding{T2A}\selectfont\`\cyrg}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER GJE
    ('\u{0454}', r#"{\fontencoding{T2A}\selectfont\cyrie}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER UKRAINIAN IE
    ('\u{0455}', r#"{\fontencoding{T2A}\selectfont\cyrdze}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER DZE
    ('\u{0456}', r#"{\fontencoding{T2A}\selectfont\cyrii}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER BYELORUSSIAN-UKRAINIAN I
    ('\u{0457}', r#"{\fontencoding{T2A}\selectfont\cyryi}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER YI
    ('\u{0458}', r#"{\fontencoding{T2A}\selectfont\cyrje}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER JE
    ('\u{0459}', r#"{\fontencoding{T2A}\selectfont\cyrlje}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER LJE
    ('\u{045A}', r#"{\fontencoding{T2A}\selectfont\cyrnje}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER NJE
    ('\u{045B}', r#"{\fontencoding{T2A}\selectfont\cyrtshe}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER TSHE
    ('\u{045C}', r#"{\fontencoding{T2A}\selectfont\`\cyrk}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER KJE
    ('\u{045D}', r#"{\fontencoding{T2A}\selectfont\`\cyri}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER I WITH GRAVE
    ('\u{045E}', r#"{\fontencoding{T2A}\selectfont\cyrushrt}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER SHORT U
    ('\u{045F}', r#"{\fontencoding{T2A}\selectfont\cyrdzhe}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER DZHE
    ('\u{0460}', r#"{\fontencoding{T2D}\selectfont\CYROMGA}"#, FONTENC_T2D), // CYRILLIC CAPITAL LETTER OMEGA
    ('\u{0461}', r#"{\fontencoding{T2D}\selectfont\cyromga}"#, FONTENC_T2D), // CYRILLIC SMALL LETTER OMEGA
    ('\u{0462}', r#"{\fontencoding{X2}\selectfont\CYRYAT}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER YAT
    ('\u{0463}', r#"{\fontencoding{X2}\selectfont\cyryat}"#, FONTENC_X2), // CYRILLIC SMALL LETTER YAT
    ('\u{0464}', r#"{\fontencoding{T2D}\selectfont\CYRIOTEST}"#, FONTENC_T2D), // CYRILLIC CAPITAL LETTER IOTIFIED E
    ('\u{0465}', r#"{\fontencoding{T2D}\selectfont\cyriotest}"#, FONTENC_T2D), // CYRILLIC SMALL LETTER IOTIFIED E
    ('\u{0466}', r#"{\fontencoding{T2D}\selectfont\CYRLYUS}"#, FONTENC_T2D), // CYRILLIC CAPITAL LETTER LITTLE YUS
    ('\u{0467}', r#"{\fontencoding{T2D}\selectfont\cyrlyus}"#, FONTENC_T2D), // CYRILLIC SMALL LETTER LITTLE YUS
    ('\u{0468}', r#"{\fontencoding{T2D}\selectfont\CYRIOTLYUS}"#, FONTENC_T2D), // CYRILLIC CAPITAL LETTER IOTIFIED LITTLE YUS
    ('\u{0469}', r#"{\fontencoding{T2D}\selectfont\cyriotlyus}"#, FONTENC_T2D), // CYRILLIC SMALL LETTER IOTIFIED LITTLE YUS
    ('\u{046A}', r#"{\fontencoding{X2}\selectfont\CYRBYUS}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER BIG YUS
    ('\u{046B}', r#"{\fontencoding{X2}\selectfont\cyrbyus}"#, FONTENC_X2), // CYRILLIC SMALL LETTER BIG YUS
    ('\u{046C}', r#"{\fontencoding{T2D}\selectfont\CYRIOTBYUS}"#, FONTENC_T2D), // CYRILLIC CAPITAL LETTER IOTIFIED BIG YUS
    ('\u{046D}', r#"{\fontencoding{T2D}\selectfont\cyriotbyus}"#, FONTENC_T2D), // CYRILLIC SMALL LETTER IOTIFIED BIG YUS
    ('\u{046E}', r#"{\fontencoding{T2D}\selectfont\CYRKSI}"#, FONTENC_T2D), // CYRILLIC CAPITAL LETTER KSI
    ('\u{046F}', r#"{\fontencoding{T2D}\selectfont\cyrksi}"#, FONTENC_T2D), // CYRILLIC SMALL LETTER KSI
    ('\u{0470}', r#"{\fontencoding{T2D}\selectfont\CYRPSI}"#, FONTENC_T2D), // CYRILLIC CAPITAL LETTER PSI
    ('\u{0471}', r#"{\fontencoding{T2D}\selectfont\cyrpsi}"#, FONTENC_T2D), // CYRILLIC SMALL LETTER PSI
    ('\u{0472}', r#"{\fontencoding{OT2}\selectfont\CYRFITA}"#, FONTENC_OT2), // CYRILLIC CAPITAL LETTER FITA
    ('\u{0473}', r#"{\fontencoding{OT2}\selectfont\cyrfita}"#, FONTENC_OT2), // CYRILLIC SMALL LETTER FITA
    ('\u{0474}', r#"{\fontencoding{X2}\selectfont\CYRIZH}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER IZHITSA
    ('\u{0475}', r#"{\fontencoding{X2}\selectfont\cyrizh}"#, FONTENC_X2), // CYRILLIC SMALL LETTER IZHITSA
    ('\u{0476}', r#"{\fontencoding{X2}\selectfont\C\CYRIZH}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER IZHITSA WITH DOUBLE GRAVE ACCENT
    ('\u{0477}', r#"{\fontencoding{X2}\selectfont\C\cyrizh}"#, FONTENC_X2), // CYRILLIC SMALL LETTER IZHITSA WITH DOUBLE GRAVE ACCENT
    ('\u{0478}', r#"{\fontencoding{T2D}\selectfont\CYRUK}"#, FONTENC_T2D), // CYRILLIC CAPITAL LETTER UK
    ('\u{0479}', r#"{\fontencoding{T2D}\selectfont\cyruk}"#, FONTENC_T2D), // CYRILLIC SMALL LETTER UK
    ('\u{047A}', r#"{\fontencoding{T2D}\selectfont\CYROMRND}"#, FONTENC_T2D), // CYRILLIC CAPITAL LETTER ROUND OMEGA
    ('\u{047B}', r#"{\fontencoding{T2D}\selectfont\cyromrnd}"#, FONTENC_T2D), // CYRILLIC SMALL LETTER ROUND OMEGA
    ('\u{047C}', r#"{\fontencoding{T2D}\selectfont\CYROMTLO}"#, FONTENC_T2D), // CYRILLIC CAPITAL LETTER OMEGA WITH TITLO
    ('\u{047D}', r#"{\fontencoding{T2D}\selectfont\cyromtlo}"#, FONTENC_T2D), // CYRILLIC SMALL LETTER OMEGA WITH TITLO
    ('\u{047E}', r#"{\fontencoding{T2D}\selectfont\CYROT}"#, FONTENC_T2D), // CYRILLIC CAPITAL LETTER OT
    ('\u{047F}', r#"{\fontencoding{T2D}\selectfont\cyrot}"#, FONTENC_T2D), // CYRILLIC SMALL LETTER OT
    ('\u{0480}', r#"{\fontencoding{T2D}\selectfont\CYRKOPPA}"#, FONTENC_T2D), // CYRILLIC CAPITAL LETTER KOPPA
    ('\u{0481}', r#"{\fontencoding{T2D}\selectfont\cyrkoppa}"#, FONTENC_T2D), // CYRILLIC SMALL LETTER KOPPA
    ('\u{0482}', r#"{\fontencoding{T2D}\selectfont\flmCyrThousands}"#, CYR_THOUSANDS), // CYRILLIC THOUSANDS SIGN
    ('\u{048C}', r#"{\fontencoding{T2C}\selectfont\CYRSEMISFTSN}"#, FONTENC_T2C), // CYRILLIC CAPITAL LETTER SEMISOFT SIGN
    ('\u{048D}', r#"{\fontencoding{T2C}\selectfont\cyrsemisftsn}"#, FONTENC_T2C), // CYRILLIC SMALL LETTER SEMISOFT SIGN
    ('\u{048E}', r#"{\fontencoding{T2C}\selectfont\CYRRTICK}"#, FONTENC_T2C), // CYRILLIC CAPITAL LETTER ER WITH TICK
    ('\u{048F}', r#"{\fontencoding{T2C}\selectfont\cyrrtick}"#, FONTENC_T2C), // CYRILLIC SMALL LETTER ER WITH TICK
    ('\u{0490}', r#"{\fontencoding{T2A}\selectfont\CYRGUP}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER GHE WITH UPTURN
    ('\u{0491}', r#"{\fontencoding{T2A}\selectfont\cyrgup}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER GHE WITH UPTURN
    ('\u{0492}', r#"{\fontencoding{T2A}\selectfont\CYRGHCRS}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER GHE WITH STROKE
    ('\u{0493}', r#"{\fontencoding{T2A}\selectfont\cyrghcrs}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER GHE WITH STROKE
    ('\u{0494}', r#"{\fontencoding{X2}\selectfont\CYRGHK}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER GHE WITH MIDDLE HOOK
    ('\u{0495}', r#"{\fontencoding{X2}\selectfont\cyrghk}"#, FONTENC_X2), // CYRILLIC SMALL LETTER GHE WITH MIDDLE HOOK
    ('\u{0496}', r#"{\fontencoding{T2A}\selectfont\CYRZHDSC}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ZHE WITH DESCENDER
    ('\u{0497}', r#"{\fontencoding{T2A}\selectfont\cyrzhdsc}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER ZHE WITH DESCENDER
    ('\u{0498}', r#"{\fontencoding{T2A}\selectfont\CYRZDSC}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ZE WITH DESCENDER
    ('\u{0499}', r#"{\fontencoding{T2A}\selectfont\cyrzdsc}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER ZE WITH DESCENDER
    ('\u{049A}', r#"{\fontencoding{T2A}\selectfont\CYRKDSC}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER KA WITH DESCENDER
    ('\u{049B}', r#"{\fontencoding{T2A}\selectfont\cyrkdsc}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER KA WITH DESCENDER
    ('\u{049C}', r#"{\fontencoding{T2A}\selectfont\CYRKVCRS}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER KA WITH VERTICAL STROKE
    ('\u{049D}', r#"{\fontencoding{T2A}\selectfont\cyrkvcrs}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER KA WITH VERTICAL STROKE
    ('\u{049E}', r#"{\fontencoding{X2}\selectfont\CYRKHCRS}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER KA WITH STROKE
    ('\u{049F}', r#"{\fontencoding{X2}\selectfont\cyrkhcrs}"#, FONTENC_X2), // CYRILLIC SMALL LETTER KA WITH STROKE
    ('\u{04A0}', r#"{\fontencoding{T2A}\selectfont\CYRKBEAK}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER BASHKIR KA
    ('\u{04A1}', r#"{\fontencoding{T2A}\selectfont\cyrkbeak}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER BASHKIR KA
    ('\u{04A2}', r#"{\fontencoding{T2A}\selectfont\CYRNDSC}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER EN WITH DESCENDER
    ('\u{04A3}', r#"{\fontencoding{T2A}\selectfont\cyrndsc}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER EN WITH DESCENDER
    ('\u{04A4}', r#"{\fontencoding{T2A}\selectfont\CYRNG}"#, FONTENC_T2A), // CYRILLIC CAPITAL LIGATURE EN GHE
    ('\u{04A5}', r#"{\fontencoding{T2A}\selectfont\cyrng}"#, FONTENC_T2A), // CYRILLIC SMALL LIGATURE EN GHE
    ('\u{04A6}', r#"{\fontencoding{X2}\selectfont\CYRPHK}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER PE WITH MIDDLE HOOK
    ('\u{04A7}', r#"{\fontencoding{X2}\selectfont\cyrphk}"#, FONTENC_X2), // CYRILLIC SMALL LETTER PE WITH MIDDLE HOOK
    ('\u{04A8}', r#"{\fontencoding{X2}\selectfont\CYRABHHA}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER ABKHASIAN HA
    ('\u{04A9}', r#"{\fontencoding{X2}\selectfont\cyrabhha}"#, FONTENC_X2), // CYRILLIC SMALL LETTER ABKHASIAN HA
    ('\u{04AA}', r#"{\fontencoding{T2A}\selectfont\CYRSDSC}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ES WITH DESCENDER
    ('\u{04AB}', r#"{\fontencoding{T2A}\selectfont\cyrsdsc}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER ES WITH DESCENDER
    ('\u{04AC}', r#"{\fontencoding{X2}\selectfont\CYRTDSC}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER TE WITH DESCENDER
    ('\u{04AD}', r#"{\fontencoding{X2}\selectfont\cyrtdsc}"#, FONTENC_X2), // CYRILLIC SMALL LETTER TE WITH DESCENDER
    ('\u{04AE}', r#"{\fontencoding{T2A}\selectfont\CYRY}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER STRAIGHT U
    ('\u{04AF}', r#"{\fontencoding{T2A}\selectfont\cyry}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER STRAIGHT U
    ('\u{04B0}', r#"{\fontencoding{T2A}\selectfont\CYRYHCRS}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER STRAIGHT U WITH STROKE
    ('\u{04B1}', r#"{\fontencoding{T2A}\selectfont\cyryhcrs}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER STRAIGHT U WITH STROKE
    ('\u{04B2}', r#"{\fontencoding{T2A}\selectfont\CYRHDSC}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER HA WITH DESCENDER
    ('\u{04B3}', r#"{\fontencoding{T2A}\selectfont\cyrhdsc}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER HA WITH DESCENDER
    ('\u{04B4}', r#"{\fontencoding{X2}\selectfont\CYRTETSE}"#, FONTENC_X2), // CYRILLIC CAPITAL LIGATURE TE TSE
    ('\u{04B5}', r#"{\fontencoding{X2}\selectfont\cyrtetse}"#, FONTENC_X2), // CYRILLIC SMALL LIGATURE TE TSE
    ('\u{04B6}', r#"{\fontencoding{T2A}\selectfont\CYRCHRDSC}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER CHE WITH DESCENDER
    ('\u{04B7}', r#"{\fontencoding{T2A}\selectfont\cyrchrdsc}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER CHE WITH DESCENDER
    ('\u{04B8}', r#"{\fontencoding{T2A}\selectfont\CYRCHVCRS}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER CHE WITH VERTICAL STROKE
    ('\u{04B9}', r#"{\fontencoding{T2A}\selectfont\cyrchvcrs}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER CHE WITH VERTICAL STROKE
    ('\u{04BA}', r#"{\fontencoding{T2A}\selectfont\CYRSHHA}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SHHA
    ('\u{04BB}', r#"{\fontencoding{T2A}\selectfont\cyrshha}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER SHHA
    ('\u{04BC}', r#"{\fontencoding{X2}\selectfont\CYRABHCH}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER ABKHASIAN CHE
    ('\u{04BD}', r#"{\fontencoding{X2}\selectfont\cyrabhch}"#, FONTENC_X2), // CYRILLIC SMALL LETTER ABKHASIAN CHE
    ('\u{04BE}', r#"{\fontencoding{X2}\selectfont\CYRABHCHDSC}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER ABKHASIAN CHE WITH DESCENDER
    ('\u{04BF}', r#"{\fontencoding{X2}\selectfont\cyrabhchdsc}"#, FONTENC_X2), // CYRILLIC SMALL LETTER ABKHASIAN CHE WITH DESCENDER
    ('\u{04C0}', r#"{\fontencoding{T2A}\selectfont\CYRpalochka}"#, FONTENC_T2A), // CYRILLIC LETTER PALOCHKA
    ('\u{04C1}', r#"{\fontencoding{T2A}\selectfont\U\CYRZH}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ZHE WITH BREVE
    ('\u{04C2}', r#"{\fontencoding{T2A}\selectfont\U\cyrzh}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER ZHE WITH BREVE
    ('\u{04C3}', r#"{\fontencoding{X2}\selectfont\CYRKHK}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER KA WITH HOOK
    ('\u{04C4}', r#"{\fontencoding{X2}\selectfont\cyrkhk}"#, FONTENC_X2), // CYRILLIC SMALL LETTER KA WITH HOOK
    ('\u{04C5}', r#"{\fontencoding{X2}\selectfont\CYRLDSC}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER EL WITH TAIL
    ('\u{04C6}', r#"{\fontencoding{X2}\selectfont\cyrldsc}"#, FONTENC_X2), // CYRILLIC SMALL LETTER EL WITH TAIL
    ('\u{04C7}', r#"{\fontencoding{X2}\selectfont\CYRNHK}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER EN WITH HOOK
    ('\u{04C8}', r#"{\fontencoding{X2}\selectfont\cyrnhk}"#, FONTENC_X2), // CYRILLIC SMALL LETTER EN WITH HOOK
    ('\u{04CB}', r#"{\fontencoding{X2}\selectfont\CYRCHLDSC}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER KHAKASSIAN CHE
    ('\u{04CC}', r#"{\fontencoding{X2}\selectfont\cyrchldsc}"#, FONTENC_X2), // CYRILLIC SMALL LETTER KHAKASSIAN CHE
    ('\u{04CD}', r#"{\fontencoding{X2}\selectfont\CYRMDSC}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER EM WITH TAIL
    ('\u{04CE}', r#"{\fontencoding{X2}\selectfont\cyrmdsc}"#, FONTENC_X2), // CYRILLIC SMALL LETTER EM WITH TAIL
    ('\u{04D0}', r#"{\fontencoding{T2A}\selectfont\U\CYRA}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER A WITH BREVE
    ('\u{04D1}', r#"{\fontencoding{T2A}\selectfont\U\cyra}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER A WITH BREVE
    ('\u{04D2}', r#"{\fontencoding{T2A}\selectfont\"\CYRA}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER A WITH DIAERESIS
    ('\u{04D3}', r#"{\fontencoding{T2A}\selectfont\"\cyra}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER A WITH DIAERESIS
    ('\u{04D4}', r#"{\fontencoding{T2A}\selectfont\CYRAE}"#, FONTENC_T2A), // CYRILLIC CAPITAL LIGATURE A IE
    ('\u{04D5}', r#"{\fontencoding{T2A}\selectfont\cyrae}"#, FONTENC_T2A), // CYRILLIC SMALL LIGATURE A IE
    ('\u{04D6}', r#"{\fontencoding{T2A}\selectfont\U\CYRE}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER IE WITH BREVE
    ('\u{04D7}', r#"{\fontencoding{T2A}\selectfont\U\cyre}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER IE WITH BREVE
    ('\u{04D8}', r#"{\fontencoding{T2A}\selectfont\CYRSCHWA}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SCHWA
    ('\u{04D9}', r#"{\fontencoding{T2A}\selectfont\cyrschwa}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER SCHWA
    ('\u{04DA}', r#"{\fontencoding{T2A}\selectfont\"\CYRSCHWA}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER SCHWA WITH DIAERESIS
    ('\u{04DB}', r#"{\fontencoding{T2A}\selectfont\"\cyrschwa}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER SCHWA WITH DIAERESIS
    ('\u{04DC}', r#"{\fontencoding{T2A}\selectfont\"\CYRZH}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ZHE WITH DIAERESIS
    ('\u{04DD}', r#"{\fontencoding{T2A}\selectfont\"\cyrzh}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER ZHE WITH DIAERESIS
    ('\u{04DE}', r#"{\fontencoding{T2A}\selectfont\"\CYRZ}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER ZE WITH DIAERESIS
    ('\u{04DF}', r#"{\fontencoding{T2A}\selectfont\"\cyrz}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER ZE WITH DIAERESIS
    ('\u{04E0}', r#"{\fontencoding{X2}\selectfont\CYRABHDZE}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER ABKHASIAN DZE
    ('\u{04E1}', r#"{\fontencoding{X2}\selectfont\cyrabhdze}"#, FONTENC_X2), // CYRILLIC SMALL LETTER ABKHASIAN DZE
    ('\u{04E2}', r#"{\fontencoding{T2A}\selectfont\=\CYRI}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER I WITH MACRON
    ('\u{04E3}', r#"{\fontencoding{T2A}\selectfont\=\cyri}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER I WITH MACRON
    ('\u{04E4}', r#"{\fontencoding{T2A}\selectfont\"\CYRI}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER I WITH DIAERESIS
    ('\u{04E5}', r#"{\fontencoding{T2A}\selectfont\"\cyri}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER I WITH DIAERESIS
    ('\u{04E6}', r#"{\fontencoding{T2A}\selectfont\"\CYRO}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER O WITH DIAERESIS
    ('\u{04E7}', r#"{\fontencoding{T2A}\selectfont\"\cyro}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER O WITH DIAERESIS
    ('\u{04E8}', r#"{\fontencoding{T2A}\selectfont\CYROTLD}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER BARRED O
    ('\u{04E9}', r#"{\fontencoding{T2A}\selectfont\cyrotld}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER BARRED O
    ('\u{04EC}', r#"{\fontencoding{T2A}\selectfont\"\CYREREV}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER E WITH DIAERESIS
    ('\u{04ED}', r#"{\fontencoding{T2A}\selectfont\"\cyrerev}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER E WITH DIAERESIS
    ('\u{04EE}', r#"{\fontencoding{T2A}\selectfont\=\CYRU}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER U WITH MACRON
    ('\u{04EF}', r#"{\fontencoding{T2A}\selectfont\=\cyru}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER U WITH MACRON
    ('\u{04F0}', r#"{\fontencoding{T2A}\selectfont\"\CYRU}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER U WITH DIAERESIS
    ('\u{04F1}', r#"{\fontencoding{T2A}\selectfont\"\cyru}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER U WITH DIAERESIS
    ('\u{04F2}', r#"{\fontencoding{T2A}\selectfont\H\CYRU}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER U WITH DOUBLE ACUTE
    ('\u{04F3}', r#"{\fontencoding{T2A}\selectfont\H\cyru}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER U WITH DOUBLE ACUTE
    ('\u{04F4}', r#"{\fontencoding{T2A}\selectfont\"\CYRCH}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER CHE WITH DIAERESIS
    ('\u{04F5}', r#"{\fontencoding{T2A}\selectfont\"\cyrch}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER CHE WITH DIAERESIS
    ('\u{04F6}', r#"{\fontencoding{X2}\selectfont\CYRGDSC}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER GHE WITH DESCENDER
    ('\u{04F7}', r#"{\fontencoding{X2}\selectfont\cyrgdsc}"#, FONTENC_X2), // CYRILLIC SMALL LETTER GHE WITH DESCENDER
    ('\u{04F8}', r#"{\fontencoding{T2A}\selectfont\"\CYRERY}"#, FONTENC_T2A), // CYRILLIC CAPITAL LETTER YERU WITH DIAERESIS
    ('\u{04F9}', r#"{\fontencoding{T2A}\selectfont\"\cyrery}"#, FONTENC_T2A), // CYRILLIC SMALL LETTER YERU WITH DIAERESIS
    ('\u{04FA}', r#"{\fontencoding{T2B}\selectfont\CYRGDSCHCRS}"#, FONTENC_T2B), // CYRILLIC CAPITAL LETTER GHE WITH STROKE AND HOOK
    ('\u{04FB}', r#"{\fontencoding{T2B}\selectfont\cyrgdschcrs}"#, FONTENC_T2B), // CYRILLIC SMALL LETTER GHE WITH STROKE AND HOOK
    ('\u{04FC}', r#"{\fontencoding{X2}\selectfont\CYRHHK}"#, FONTENC_X2), // CYRILLIC CAPITAL LETTER HA WITH HOOK
    ('\u{04FD}', r#"{\fontencoding{X2}\selectfont\cyrhhk}"#, FONTENC_X2), // CYRILLIC SMALL LETTER HA WITH HOOK
    ('\u{04FE}', r#"{\fontencoding{T2B}\selectfont\CYRHHCRS}"#, FONTENC_T2B), // CYRILLIC CAPITAL LETTER HA WITH STROKE
    ('\u{04FF}', r#"{\fontencoding{T2B}\selectfont\cyrhhcrs}"#, FONTENC_T2B), // CYRILLIC SMALL LETTER HA WITH STROKE
    ('\u{0E3F}', r#"\textbaht"#, BUILTINS), // THAI CURRENCY SYMBOL BAHT
    ('\u{2000}', r#"\enskip"#, BUILTINS), // EN QUAD
    ('\u{2001}', r#"\quad"#, BUILTINS), // EM QUAD
    ('\u{2002}', r#"\enskip"#, BUILTINS), // EN SPACE
    ('\u{2003}', r#"\quad"#, BUILTINS), // EM SPACE
    ('\u{2004}', r#"\hspace{0.33em}"#, BUILTINS), // THREE-PER-EM SPACE
    ('\u{2005}', r#"\hspace{0.25em}"#, BUILTINS), // FOUR-PER-EM SPACE
    ('\u{2006}', r#"\hspace{0.167em}"#, BUILTINS), // SIX-PER-EM SPACE
    ('\u{2007}', r#"~"#, BUILTINS), // FIGURE SPACE
    ('\u{2008}', r#"\;"#, BUILTINS), // PUNCTUATION SPACE
    ('\u{2009}', r#"\,"#, BUILTINS), // THIN SPACE
    ('\u{200A}', r#"\hspace{1pt}"#, BUILTINS), // HAIR SPACE
    ('\u{200C}', r#"\textcompwordmark"#, BUILTINS), // ZERO WIDTH NON-JOINER
    ('\u{2010}', r#"-"#, BUILTINS), // HYPHEN
    ('\u{2011}', r#"\nobreakdash-"#, AMSMATH), // NON-BREAKING HYPHEN
    ('\u{2012}', r#"-"#, BUILTINS), // FIGURE DASH
    ('\u{2013}', r#"\textendash"#, BUILTINS), // EN DASH
    ('\u{2014}', r#"\textemdash"#, BUILTINS), // EM DASH
    ('\u{2015}', r#"\textemdash"#, BUILTINS), // HORIZONTAL BAR
    ('\u{2016}', r#"\ensuremath{\Vert}"#, BUILTINS), // DOUBLE VERTICAL LINE
    ('\u{2018}', r#"\textquoteleft"#, BUILTINS), // LEFT SINGLE QUOTATION MARK
    ('\u{2019}', r#"\textquoteright"#, BUILTINS), // RIGHT SINGLE QUOTATION MARK
    ('\u{201A}', r#"\quotesinglbase"#, FONTENC_T1), // SINGLE LOW-9 QUOTATION MARK
    ('\u{201C}', r#"\textquotedblleft"#, BUILTINS), // LEFT DOUBLE QUOTATION MARK
    ('\u{201D}', r#"\textquotedblright"#, BUILTINS), // RIGHT DOUBLE QUOTATION MARK
    ('\u{201E}', r#"\quotedblbase"#, FONTENC_T1), // DOUBLE LOW-9 QUOTATION MARK
    ('\u{2020}', r#"\textdagger"#, BUILTINS), // DAGGER
    ('\u{2021}', r#"\textdaggerdbl"#, BUILTINS), // DOUBLE DAGGER
    ('\u{2022}', r#"\textbullet"#, BUILTINS), // BULLET
    ('\u{2024}', r#"."#, BUILTINS), // ONE DOT LEADER
    ('\u{2025}', r#".."#, BUILTINS), // TWO DOT LEADER
    ('\u{2026}', r#"\textellipsis"#, BUILTINS), // HORIZONTAL ELLIPSIS
    ('\u{2030}', r#"\textperthousand"#, BUILTINS), // PER MILLE SIGN
    ('\u{2031}', r#"\textpertenthousand"#, BUILTINS), // PER TEN THOUSAND SIGN
    ('\u{2032}', r#"'"#, BUILTINS), // PRIME
    ('\u{2033}', r#"''"#, BUILTINS), // DOUBLE PRIME
    ('\u{2034}', r#"'''"#, BUILTINS), // TRIPLE PRIME
    ('\u{2035}', r#"\ensuremath{\backprime}"#, AMSSYMB), // REVERSED PRIME
    ('\u{2039}', r#"\guilsinglleft"#, FONTENC_T1), // SINGLE LEFT-POINTING ANGLE QUOTATION MARK
    ('\u{203A}', r#"\guilsinglright"#, FONTENC_T1), // SINGLE RIGHT-POINTING ANGLE QUOTATION MARK
    ('\u{203B}', r#"\textreferencemark"#, BUILTINS), // REFERENCE MARK
    ('\u{203D}', r#"\textinterrobang"#, BUILTINS), // INTERROBANG
    ('\u{2044}', r#"\textfractionsolidus"#, BUILTINS), // FRACTION SLASH
    ('\u{204E}', r#"\textasteriskcentered"#, BUILTINS), // LOW ASTERISK
    ('\u{2052}', r#"\textdiscount"#, BUILTINS), // COMMERCIAL MINUS SIGN
    ('\u{2057}', r#"''''"#, BUILTINS), // QUADRUPLE PRIME
    ('\u{205F}', r#"\hspace{0.22em}"#, BUILTINS), // MEDIUM MATHEMATICAL SPACE
    ('\u{2060}', r#"\nolinebreak"#, BUILTINS), // WORD JOINER
    ('\u{2061}', r#""#, BUILTINS), // FUNCTION APPLICATION
    ('\u{2070}', r#"\ensuremath{^0}"#, BUILTINS), // SUPERSCRIPT ZERO
    ('\u{2071}', r#"\ensuremath{^i}"#, BUILTINS), // SUPERSCRIPT LATIN SMALL LETTER I
    ('\u{2074}', r#"\ensuremath{^4}"#, BUILTINS), // SUPERSCRIPT FOUR
    ('\u{2075}', r#"\ensuremath{^5}"#, BUILTINS), // SUPERSCRIPT FIVE
    ('\u{2076}', r#"\ensuremath{^6}"#, BUILTINS), // SUPERSCRIPT SIX
    ('\u{2077}', r#"\ensuremath{^7}"#, BUILTINS), // SUPERSCRIPT SEVEN
    ('\u{2078}', r#"\ensuremath{^8}"#, BUILTINS), // SUPERSCRIPT EIGHT
    ('\u{2079}', r#"\ensuremath{^9}"#, BUILTINS), // SUPERSCRIPT NINE
    ('\u{207A}', r#"\ensuremath{^+}"#, BUILTINS), // SUPERSCRIPT PLUS SIGN
    ('\u{207B}', r#"\ensuremath{^-}"#, BUILTINS), // SUPERSCRIPT MINUS
    ('\u{207C}', r#"\ensuremath{^=}"#, BUILTINS), // SUPERSCRIPT EQUALS SIGN
    ('\u{207D}', r#"\ensuremath{^(}"#, BUILTINS), // SUPERSCRIPT LEFT PARENTHESIS
    ('\u{207E}', r#"\ensuremath{^)}"#, BUILTINS), // SUPERSCRIPT RIGHT PARENTHESIS
    ('\u{207F}', r#"\ensuremath{^n}"#, BUILTINS), // SUPERSCRIPT LATIN SMALL LETTER N
    ('\u{2080}', r#"\ensuremath{_0}"#, BUILTINS), // SUBSCRIPT ZERO
    ('\u{2081}', r#"\ensuremath{_1}"#, BUILTINS), // SUBSCRIPT ONE
    ('\u{2082}', r#"\ensuremath{_2}"#, BUILTINS), // SUBSCRIPT TWO
    ('\u{2083}', r#"\ensuremath{_3}"#, BUILTINS), // SUBSCRIPT THREE
    ('\u{2084}', r#"\ensuremath{_4}"#, BUILTINS), // SUBSCRIPT FOUR
    ('\u{2085}', r#"\ensuremath{_5}"#, BUILTINS), // SUBSCRIPT FIVE
    ('\u{2086}', r#"\ensuremath{_6}"#, BUILTINS), // SUBSCRIPT SIX
    ('\u{2087}', r#"\ensuremath{_7}"#, BUILTINS), // SUBSCRIPT SEVEN
    ('\u{2088}', r#"\ensuremath{_8}"#, BUILTINS), // SUBSCRIPT EIGHT
    ('\u{2089}', r#"\ensuremath{_9}"#, BUILTINS), // SUBSCRIPT NINE
    ('\u{208A}', r#"\ensuremath{_+}"#, BUILTINS), // SUBSCRIPT PLUS SIGN
    ('\u{208B}', r#"\ensuremath{_-}"#, BUILTINS), // SUBSCRIPT MINUS
    ('\u{208C}', r#"\ensuremath{_=}"#, BUILTINS), // SUBSCRIPT EQUALS SIGN
    ('\u{208D}', r#"\ensuremath{_(}"#, BUILTINS), // SUBSCRIPT LEFT PARENTHESIS
    ('\u{208E}', r#"\ensuremath{_)}"#, BUILTINS), // SUBSCRIPT RIGHT PARENTHESIS
    ('\u{2090}', r#"\ensuremath{_a}"#, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER A
    ('\u{2091}', r#"\ensuremath{_e}"#, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER E
    ('\u{2092}', r#"\ensuremath{_o}"#, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER O
    ('\u{2093}', r#"\ensuremath{_x}"#, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER X
    ('\u{2095}', r#"\ensuremath{_h}"#, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER H
    ('\u{2096}', r#"\ensuremath{_k}"#, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER K
    ('\u{2097}', r#"\ensuremath{_l}"#, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER L
    ('\u{2098}', r#"\ensuremath{_m}"#, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER M
    ('\u{2099}', r#"\ensuremath{_n}"#, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER N
    ('\u{209A}', r#"\ensuremath{_p}"#, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER P
    ('\u{209B}', r#"\ensuremath{_s}"#, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER S
    ('\u{209C}', r#"\ensuremath{_t}"#, BUILTINS), // LATIN SUBSCRIPT SMALL LETTER T
    ('\u{20A1}', r#"\textcolonmonetary"#, BUILTINS), // COLON SIGN
    ('\u{20A4}', r#"\textlira"#, BUILTINS), // LIRA SIGN
    ('\u{20A6}', r#"\textnaira"#, BUILTINS), // NAIRA SIGN
    ('\u{20A9}', r#"\textwon"#, BUILTINS), // WON SIGN
    ('\u{20AB}', r#"\textdong"#, BUILTINS), // DONG SIGN
    ('\u{20AC}', r#"\texteuro"#, BUILTINS), // EURO SIGN
    ('\u{20B1}', r#"\textpeso"#, BUILTINS), // PESO SIGN
    ('\u{2102}', r#"\ensuremath{\mathbb{C}}"#, AMSSYMB), // DOUBLE-STRUCK CAPITAL C
    ('\u{2103}', r#"\textcelsius"#, BUILTINS), // DEGREE CELSIUS
    ('\u{2109}', r#"\ensuremath{^\circ}F"#, BUILTINS), // DEGREE FAHRENHEIT
    ('\u{210A}', r#"\ensuremath{\flmScr{g}}"#, SCRIPT), // SCRIPT SMALL G
    ('\u{210B}', r#"\ensuremath{\mathscr{H}}"#, MATHRSFS), // SCRIPT CAPITAL H
    ('\u{210C}', r#"\ensuremath{\mathfrak{H}}"#, AMSSYMB), // BLACK-LETTER CAPITAL H
    ('\u{210D}', r#"\ensuremath{\mathbb{H}}"#, AMSSYMB), // DOUBLE-STRUCK CAPITAL H
    ('\u{210E}', r#"\ensuremath{h}"#, BUILTINS), // PLANCK CONSTANT
    ('\u{210F}', r#"\ensuremath{\hbar}"#, BUILTINS), // PLANCK CONSTANT OVER TWO PI
    ('\u{2110}', r#"\ensuremath{\mathscr{I}}"#, MATHRSFS), // SCRIPT CAPITAL I
    ('\u{2111}', r#"\ensuremath{\mathfrak{I}}"#, AMSSYMB), // BLACK-LETTER CAPITAL I
    ('\u{2112}', r#"\ensuremath{\mathscr{L}}"#, MATHRSFS), // SCRIPT CAPITAL L
    ('\u{2113}', r#"\ensuremath{\ell}"#, BUILTINS), // SCRIPT SMALL L
    ('\u{2115}', r#"\ensuremath{\mathbb{N}}"#, AMSSYMB), // DOUBLE-STRUCK CAPITAL N
    ('\u{2116}', r#"\textnumero"#, BUILTINS), // NUMERO SIGN
    ('\u{2117}', r#"\textcircledP"#, BUILTINS), // SOUND RECORDING COPYRIGHT
    ('\u{2118}', r#"\ensuremath{\wp}"#, BUILTINS), // SCRIPT CAPITAL P
    ('\u{2119}', r#"\ensuremath{\mathbb{P}}"#, AMSSYMB), // DOUBLE-STRUCK CAPITAL P
    ('\u{211A}', r#"\ensuremath{\mathbb{Q}}"#, AMSSYMB), // DOUBLE-STRUCK CAPITAL Q
    ('\u{211B}', r#"\ensuremath{\mathscr{R}}"#, MATHRSFS), // SCRIPT CAPITAL R
    ('\u{211C}', r#"\ensuremath{\mathfrak{R}}"#, AMSSYMB), // BLACK-LETTER CAPITAL R
    ('\u{211D}', r#"\ensuremath{\mathbb{R}}"#, AMSSYMB), // DOUBLE-STRUCK CAPITAL R
    ('\u{211E}', r#"\textrecipe"#, BUILTINS), // PRESCRIPTION TAKE
    ('\u{2120}', r#"\textservicemark"#, BUILTINS), // SERVICE MARK
    ('\u{2122}', r#"\texttrademark"#, BUILTINS), // TRADE MARK SIGN
    ('\u{2124}', r#"\ensuremath{\mathbb{Z}}"#, AMSSYMB), // DOUBLE-STRUCK CAPITAL Z
    ('\u{2126}', r#"\textohm"#, BUILTINS), // OHM SIGN
    ('\u{2127}', r#"\textmho"#, BUILTINS), // INVERTED OHM SIGN
    ('\u{2128}', r#"\ensuremath{\mathfrak{Z}}"#, AMSSYMB), // BLACK-LETTER CAPITAL Z
    ('\u{212A}', r#"K"#, BUILTINS), // KELVIN SIGN
    ('\u{212B}', r#"\r{A}"#, BUILTINS), // ANGSTROM SIGN
    ('\u{212C}', r#"\ensuremath{\mathscr{B}}"#, MATHRSFS), // SCRIPT CAPITAL B
    ('\u{212D}', r#"\ensuremath{\mathfrak{C}}"#, AMSSYMB), // BLACK-LETTER CAPITAL C
    ('\u{212E}', r#"\textestimated"#, BUILTINS), // ESTIMATED SYMBOL
    ('\u{212F}', r#"\ensuremath{\flmScr{e}}"#, SCRIPT), // SCRIPT SMALL E
    ('\u{2130}', r#"\ensuremath{\mathscr{E}}"#, MATHRSFS), // SCRIPT CAPITAL E
    ('\u{2131}', r#"\ensuremath{\mathscr{F}}"#, MATHRSFS), // SCRIPT CAPITAL F
    ('\u{2133}', r#"\ensuremath{\mathscr{M}}"#, MATHRSFS), // SCRIPT CAPITAL M
    ('\u{2134}', r#"\ensuremath{\flmScr{o}}"#, SCRIPT), // SCRIPT SMALL O
    ('\u{2135}', r#"\ensuremath{\aleph}"#, BUILTINS), // ALEF SYMBOL
    ('\u{2136}', r#"\ensuremath{\beth}"#, AMSSYMB), // BET SYMBOL
    ('\u{2137}', r#"\ensuremath{\gimel}"#, AMSSYMB), // GIMEL SYMBOL
    ('\u{2138}', r#"\ensuremath{\daleth}"#, AMSSYMB), // DALET SYMBOL
    ('\u{2153}', r#"\nicefrac{1}{3}"#, NICEFRAC), // VULGAR FRACTION ONE THIRD
    ('\u{2154}', r#"\nicefrac{2}{3}"#, NICEFRAC), // VULGAR FRACTION TWO THIRDS
    ('\u{2155}', r#"\nicefrac{1}{5}"#, NICEFRAC), // VULGAR FRACTION ONE FIFTH
    ('\u{2156}', r#"\nicefrac{2}{5}"#, NICEFRAC), // VULGAR FRACTION TWO FIFTHS
    ('\u{2157}', r#"\nicefrac{3}{5}"#, NICEFRAC), // VULGAR FRACTION THREE FIFTHS
    ('\u{2158}', r#"\nicefrac{4}{5}"#, NICEFRAC), // VULGAR FRACTION FOUR FIFTHS
    ('\u{2159}', r#"\nicefrac{1}{6}"#, NICEFRAC), // VULGAR FRACTION ONE SIXTH
    ('\u{215A}', r#"\nicefrac{5}{6}"#, NICEFRAC), // VULGAR FRACTION FIVE SIXTHS
    ('\u{215B}', r#"\nicefrac{1}{8}"#, NICEFRAC), // VULGAR FRACTION ONE EIGHTH
    ('\u{215C}', r#"\nicefrac{3}{8}"#, NICEFRAC), // VULGAR FRACTION THREE EIGHTHS
    ('\u{215D}', r#"\nicefrac{5}{8}"#, NICEFRAC), // VULGAR FRACTION FIVE EIGHTHS
    ('\u{215E}', r#"\nicefrac{7}{8}"#, NICEFRAC), // VULGAR FRACTION SEVEN EIGHTHS
    ('\u{2190}', r#"\textleftarrow"#, BUILTINS), // LEFTWARDS ARROW
    ('\u{2191}', r#"\textuparrow"#, BUILTINS), // UPWARDS ARROW
    ('\u{2192}', r#"\textrightarrow"#, BUILTINS), // RIGHTWARDS ARROW
    ('\u{2193}', r#"\textdownarrow"#, BUILTINS), // DOWNWARDS ARROW
    ('\u{2194}', r#"\ensuremath{\leftrightarrow}"#, BUILTINS), // LEFT RIGHT ARROW
    ('\u{2195}', r#"\ensuremath{\updownarrow}"#, BUILTINS), // UP DOWN ARROW
    ('\u{2196}', r#"\ensuremath{\nwarrow}"#, BUILTINS), // NORTH WEST ARROW
    ('\u{2197}', r#"\ensuremath{\nearrow}"#, BUILTINS), // NORTH EAST ARROW
    ('\u{2198}', r#"\ensuremath{\searrow}"#, BUILTINS), // SOUTH EAST ARROW
    ('\u{2199}', r#"\ensuremath{\swarrow}"#, BUILTINS), // SOUTH WEST ARROW
    ('\u{219A}', r#"\ensuremath{\nleftarrow}"#, AMSSYMB), // LEFTWARDS ARROW WITH STROKE
    ('\u{219B}', r#"\ensuremath{\nrightarrow}"#, AMSSYMB), // RIGHTWARDS ARROW WITH STROKE
    ('\u{219C}', r#"\ensuremath{\flmleftwavearrow}"#, STIX), // LEFTWARDS WAVE ARROW
    ('\u{219D}', r#"\ensuremath{\flmrightwavearrow}"#, STIX), // RIGHTWARDS WAVE ARROW
    ('\u{219E}', r#"\ensuremath{\twoheadleftarrow}"#, AMSSYMB), // LEFTWARDS TWO HEADED ARROW
    ('\u{21A0}', r#"\ensuremath{\twoheadrightarrow}"#, AMSSYMB), // RIGHTWARDS TWO HEADED ARROW
    ('\u{21A2}', r#"\ensuremath{\leftarrowtail}"#, AMSSYMB), // LEFTWARDS ARROW WITH TAIL
    ('\u{21A3}', r#"\ensuremath{\rightarrowtail}"#, AMSSYMB), // RIGHTWARDS ARROW WITH TAIL
    ('\u{21A6}', r#"\ensuremath{\mapsto}"#, BUILTINS), // RIGHTWARDS ARROW FROM BAR
    ('\u{21A9}', r#"\ensuremath{\hookleftarrow}"#, BUILTINS), // LEFTWARDS ARROW WITH HOOK
    ('\u{21AA}', r#"\ensuremath{\hookrightarrow}"#, BUILTINS), // RIGHTWARDS ARROW WITH HOOK
    ('\u{21AB}', r#"\ensuremath{\looparrowleft}"#, AMSSYMB), // LEFTWARDS ARROW WITH LOOP
    ('\u{21AC}', r#"\ensuremath{\looparrowright}"#, AMSSYMB), // RIGHTWARDS ARROW WITH LOOP
    ('\u{21AD}', r#"\ensuremath{\leftrightsquigarrow}"#, AMSSYMB), // LEFT RIGHT WAVE ARROW
    ('\u{21AE}', r#"\ensuremath{\nleftrightarrow}"#, AMSSYMB), // LEFT RIGHT ARROW WITH STROKE
    ('\u{21B0}', r#"\ensuremath{\Lsh}"#, AMSSYMB), // UPWARDS ARROW WITH TIP LEFTWARDS
    ('\u{21B1}', r#"\ensuremath{\Rsh}"#, AMSSYMB), // UPWARDS ARROW WITH TIP RIGHTWARDS
    ('\u{21B6}', r#"\ensuremath{\curvearrowleft}"#, AMSSYMB), // ANTICLOCKWISE TOP SEMICIRCLE ARROW
    ('\u{21B7}', r#"\ensuremath{\curvearrowright}"#, AMSSYMB), // CLOCKWISE TOP SEMICIRCLE ARROW
    ('\u{21BA}', r#"\ensuremath{\circlearrowleft}"#, AMSSYMB), // ANTICLOCKWISE OPEN CIRCLE ARROW
    ('\u{21BB}', r#"\ensuremath{\circlearrowright}"#, AMSSYMB), // CLOCKWISE OPEN CIRCLE ARROW
    ('\u{21BC}', r#"\ensuremath{\leftharpoonup}"#, BUILTINS), // LEFTWARDS HARPOON WITH BARB UPWARDS
    ('\u{21BD}', r#"\ensuremath{\leftharpoondown}"#, BUILTINS), // LEFTWARDS HARPOON WITH BARB DOWNWARDS
    ('\u{21BE}', r#"\ensuremath{\upharpoonright}"#, AMSSYMB), // UPWARDS HARPOON WITH BARB RIGHTWARDS
    ('\u{21BF}', r#"\ensuremath{\upharpoonleft}"#, AMSSYMB), // UPWARDS HARPOON WITH BARB LEFTWARDS
    ('\u{21C0}', r#"\ensuremath{\rightharpoonup}"#, BUILTINS), // RIGHTWARDS HARPOON WITH BARB UPWARDS
    ('\u{21C1}', r#"\ensuremath{\rightharpoondown}"#, BUILTINS), // RIGHTWARDS HARPOON WITH BARB DOWNWARDS
    ('\u{21C2}', r#"\ensuremath{\downharpoonright}"#, AMSSYMB), // DOWNWARDS HARPOON WITH BARB RIGHTWARDS
    ('\u{21C3}', r#"\ensuremath{\downharpoonleft}"#, AMSSYMB), // DOWNWARDS HARPOON WITH BARB LEFTWARDS
    ('\u{21C4}', r#"\ensuremath{\rightleftarrows}"#, AMSSYMB), // RIGHTWARDS ARROW OVER LEFTWARDS ARROW
    ('\u{21C5}', r#"\ensuremath{\flmupdownarrows}"#, STIX), // UPWARDS ARROW LEFTWARDS OF DOWNWARDS ARROW
    ('\u{21C6}', r#"\ensuremath{\leftrightarrows}"#, AMSSYMB), // LEFTWARDS ARROW OVER RIGHTWARDS ARROW
    ('\u{21C7}', r#"\ensuremath{\leftleftarrows}"#, AMSSYMB), // LEFTWARDS PAIRED ARROWS
    ('\u{21C8}', r#"\ensuremath{\upuparrows}"#, AMSSYMB), // UPWARDS PAIRED ARROWS
    ('\u{21C9}', r#"\ensuremath{\rightrightarrows}"#, AMSSYMB), // RIGHTWARDS PAIRED ARROWS
    ('\u{21CA}', r#"\ensuremath{\downdownarrows}"#, AMSSYMB), // DOWNWARDS PAIRED ARROWS
    ('\u{21CB}', r#"\ensuremath{\leftrightharpoons}"#, AMSSYMB), // LEFTWARDS HARPOON OVER RIGHTWARDS HARPOON
    ('\u{21CC}', r#"\ensuremath{\rightleftharpoons}"#, BUILTINS), // RIGHTWARDS HARPOON OVER LEFTWARDS HARPOON
    ('\u{21CD}', r#"\ensuremath{\nLeftarrow}"#, AMSSYMB), // LEFTWARDS DOUBLE ARROW WITH STROKE
    ('\u{21CE}', r#"\ensuremath{\nLeftrightarrow}"#, AMSSYMB), // LEFT RIGHT DOUBLE ARROW WITH STROKE
    ('\u{21CF}', r#"\ensuremath{\nRightarrow}"#, AMSSYMB), // RIGHTWARDS DOUBLE ARROW WITH STROKE
    ('\u{21D0}', r#"\ensuremath{\Leftarrow}"#, BUILTINS), // LEFTWARDS DOUBLE ARROW
    ('\u{21D1}', r#"\ensuremath{\Uparrow}"#, BUILTINS), // UPWARDS DOUBLE ARROW
    ('\u{21D2}', r#"\ensuremath{\Rightarrow}"#, BUILTINS), // RIGHTWARDS DOUBLE ARROW
    ('\u{21D3}', r#"\ensuremath{\Downarrow}"#, BUILTINS), // DOWNWARDS DOUBLE ARROW
    ('\u{21D4}', r#"\ensuremath{\Leftrightarrow}"#, BUILTINS), // LEFT RIGHT DOUBLE ARROW
    ('\u{21D5}', r#"\ensuremath{\Updownarrow}"#, BUILTINS), // UP DOWN DOUBLE ARROW
    ('\u{21DA}', r#"\ensuremath{\Lleftarrow}"#, AMSSYMB), // LEFTWARDS TRIPLE ARROW
    ('\u{21DB}', r#"\ensuremath{\Rrightarrow}"#, AMSSYMB), // RIGHTWARDS TRIPLE ARROW
    ('\u{21DD}', r#"\ensuremath{\rightsquigarrow}"#, AMSSYMB), // RIGHTWARDS SQUIGGLE ARROW
    ('\u{21F5}', r#"\ensuremath{\flmdownuparrows}"#, STIX), // DOWNWARDS ARROW LEFTWARDS OF UPWARDS ARROW
    ('\u{2200}', r#"\ensuremath{\forall}"#, BUILTINS), // FOR ALL
    ('\u{2201}', r#"\ensuremath{\complement}"#, AMSSYMB), // COMPLEMENT
    ('\u{2202}', r#"\ensuremath{\partial}"#, BUILTINS), // PARTIAL DIFFERENTIAL
    ('\u{2203}', r#"\ensuremath{\exists}"#, BUILTINS), // THERE EXISTS
    ('\u{2204}', r#"\ensuremath{\nexists}"#, AMSSYMB), // THERE DOES NOT EXIST
    ('\u{2205}', r#"\ensuremath{\varnothing}"#, AMSSYMB), // EMPTY SET
    ('\u{2206}', r#"\ensuremath{\Delta}"#, BUILTINS), // INCREMENT
    ('\u{2207}', r#"\ensuremath{\nabla}"#, BUILTINS), // NABLA
    ('\u{2208}', r#"\ensuremath{\in}"#, BUILTINS), // ELEMENT OF
    ('\u{2209}', r#"\ensuremath{\notin}"#, BUILTINS), // NOT AN ELEMENT OF
    ('\u{220A}', r#"\ensuremath{\in}"#, BUILTINS), // SMALL ELEMENT OF
    ('\u{220B}', r#"\ensuremath{\ni}"#, BUILTINS), // CONTAINS AS MEMBER
    ('\u{220C}', r#"\ensuremath{\not\ni}"#, BUILTINS), // DOES NOT CONTAIN AS MEMBER
    ('\u{220D}', r#"\ensuremath{\ni}"#, BUILTINS), // SMALL CONTAINS AS MEMBER
    ('\u{220E}', r#"\ensuremath{\blacksquare}"#, AMSSYMB), // END OF PROOF
    ('\u{220F}', r#"\ensuremath{\prod}"#, BUILTINS), // N-ARY PRODUCT
    ('\u{2210}', r#"\ensuremath{\coprod}"#, BUILTINS), // N-ARY COPRODUCT
    ('\u{2211}', r#"\ensuremath{\sum}"#, BUILTINS), // N-ARY SUMMATION
    ('\u{2212}', r#"\ensuremath{-}"#, BUILTINS), // MINUS SIGN
    ('\u{2213}', r#"\ensuremath{\mp}"#, BUILTINS), // MINUS-OR-PLUS SIGN
    ('\u{2214}', r#"\ensuremath{\dotplus}"#, AMSSYMB), // DOT PLUS
    ('\u{2215}', r#"\ensuremath{/}"#, BUILTINS), // DIVISION SLASH
    ('\u{2216}', r#"\ensuremath{\smallsetminus}"#, AMSSYMB), // SET MINUS
    ('\u{2217}', r#"\ensuremath{*}"#, BUILTINS), // ASTERISK OPERATOR
    ('\u{2218}', r#"\ensuremath{\circ}"#, BUILTINS), // RING OPERATOR
    ('\u{2219}', r#"\ensuremath{\bullet}"#, BUILTINS), // BULLET OPERATOR
    ('\u{221A}', r#"\ensuremath{\sqrt{}}"#, BUILTINS), // SQUARE ROOT
    ('\u{221B}', r#"\ensuremath{\sqrt[3]{}}"#, BUILTINS), // CUBE ROOT
    ('\u{221C}', r#"\ensuremath{\sqrt[4]{}}"#, BUILTINS), // FOURTH ROOT
    ('\u{221D}', r#"\ensuremath{\propto}"#, BUILTINS), // PROPORTIONAL TO
    ('\u{221E}', r#"\ensuremath{\infty}"#, BUILTINS), // INFINITY
    ('\u{221F}', r#"\ensuremath{\flmrightangle}"#, STIX), // RIGHT ANGLE
    ('\u{2220}', r#"\ensuremath{\angle}"#, BUILTINS), // ANGLE
    ('\u{2221}', r#"\ensuremath{\measuredangle}"#, AMSSYMB), // MEASURED ANGLE
    ('\u{2222}', r#"\ensuremath{\sphericalangle}"#, AMSSYMB), // SPHERICAL ANGLE
    ('\u{2223}', r#"\ensuremath{\mid}"#, BUILTINS), // DIVIDES
    ('\u{2224}', r#"\ensuremath{\nmid}"#, AMSSYMB), // DOES NOT DIVIDE
    ('\u{2225}', r#"\ensuremath{\parallel}"#, BUILTINS), // PARALLEL TO
    ('\u{2226}', r#"\ensuremath{\nparallel}"#, AMSSYMB), // NOT PARALLEL TO
    ('\u{2227}', r#"\ensuremath{\wedge}"#, BUILTINS), // LOGICAL AND
    ('\u{2228}', r#"\ensuremath{\vee}"#, BUILTINS), // LOGICAL OR
    ('\u{2229}', r#"\ensuremath{\cap}"#, BUILTINS), // INTERSECTION
    ('\u{222A}', r#"\ensuremath{\cup}"#, BUILTINS), // UNION
    ('\u{222B}', r#"\ensuremath{\int}"#, BUILTINS), // INTEGRAL
    ('\u{222C}', r#"\ensuremath{\iint}"#, AMSMATH), // DOUBLE INTEGRAL
    ('\u{222D}', r#"\ensuremath{\iiint}"#, AMSMATH), // TRIPLE INTEGRAL
    ('\u{222E}', r#"\ensuremath{\oint}"#, BUILTINS), // CONTOUR INTEGRAL
    ('\u{222F}', r#"\ensuremath{\flmoiint}"#, STIX), // SURFACE INTEGRAL
    ('\u{2230}', r#"\ensuremath{\flmoiiint}"#, STIX), // VOLUME INTEGRAL
    ('\u{2231}', r#"\ensuremath{\flmintclockwise}"#, STIX), // CLOCKWISE INTEGRAL
    ('\u{2234}', r#"\ensuremath{\therefore}"#, AMSSYMB), // THEREFORE
    ('\u{2235}', r#"\ensuremath{\because}"#, AMSSYMB), // BECAUSE
    ('\u{2236}', r#"\ensuremath{:}"#, BUILTINS), // RATIO
    ('\u{2237}', r#"\ensuremath{::}"#, BUILTINS), // PROPORTION
    ('\u{223A}', r#"\ensuremath{\mathbin{{:}\!\!{-}\!\!{:}}}"#, BUILTINS), // GEOMETRIC PROPORTION
    ('\u{223B}', r#"\ensuremath{\flmkernelcontraction}"#, STIX), // HOMOTHETIC
    ('\u{223C}', r#"\ensuremath{\sim}"#, BUILTINS), // TILDE OPERATOR
    ('\u{223D}', r#"\ensuremath{\backsim}"#, AMSSYMB), // REVERSED TILDE
    ('\u{223E}', r#"\ensuremath{\flminvlazys}"#, STIX), // INVERTED LAZY S
    ('\u{2240}', r#"\ensuremath{\wr}"#, BUILTINS), // WREATH PRODUCT
    ('\u{2241}', r#"\ensuremath{\not\sim}"#, BUILTINS), // NOT TILDE
    ('\u{2243}', r#"\ensuremath{\simeq}"#, BUILTINS), // ASYMPTOTICALLY EQUAL TO
    ('\u{2244}', r#"\ensuremath{\not\simeq}"#, BUILTINS), // NOT ASYMPTOTICALLY EQUAL TO
    ('\u{2245}', r#"\ensuremath{\cong}"#, BUILTINS), // APPROXIMATELY EQUAL TO
    ('\u{2246}', r#"\ensuremath{\flmsimneqq}"#, STIX), // APPROXIMATELY BUT NOT ACTUALLY EQUAL TO
    ('\u{2247}', r#"\ensuremath{\not\cong}"#, BUILTINS), // NEITHER APPROXIMATELY NOR ACTUALLY EQUAL TO
    ('\u{2248}', r#"\ensuremath{\approx}"#, BUILTINS), // ALMOST EQUAL TO
    ('\u{2249}', r#"\ensuremath{\not\approx}"#, BUILTINS), // NOT ALMOST EQUAL TO
    ('\u{224A}', r#"\ensuremath{\approxeq}"#, AMSSYMB), // ALMOST EQUAL OR EQUAL TO
    ('\u{224B}', r#"\ensuremath{\flmapproxident}"#, STIX), // TRIPLE TILDE
    ('\u{224C}', r#"\ensuremath{\flmbackcong}"#, STIX), // ALL EQUAL TO
    ('\u{224D}', r#"\ensuremath{\asymp}"#, BUILTINS), // EQUIVALENT TO
    ('\u{224E}', r#"\ensuremath{\Bumpeq}"#, AMSSYMB), // GEOMETRICALLY EQUIVALENT TO
    ('\u{224F}', r#"\ensuremath{\bumpeq}"#, AMSSYMB), // DIFFERENCE BETWEEN
    ('\u{2250}', r#"\ensuremath{\doteq}"#, BUILTINS), // APPROACHES THE LIMIT
    ('\u{2251}', r#"\ensuremath{\doteqdot}"#, AMSSYMB), // GEOMETRICALLY EQUAL TO
    ('\u{2252}', r#"\ensuremath{\fallingdotseq}"#, AMSSYMB), // APPROXIMATELY EQUAL TO OR THE IMAGE OF
    ('\u{2253}', r#"\ensuremath{\risingdotseq}"#, AMSSYMB), // IMAGE OF OR APPROXIMATELY EQUAL TO
    ('\u{2254}', r#"\ensuremath{:=}"#, BUILTINS), // COLON EQUALS
    ('\u{2255}', r#"\ensuremath{=:}"#, BUILTINS), // EQUALS COLON
    ('\u{2256}', r#"\ensuremath{\eqcirc}"#, AMSSYMB), // RING IN EQUAL TO
    ('\u{2257}', r#"\ensuremath{\circeq}"#, AMSSYMB), // RING EQUAL TO
    ('\u{2259}', r#"\ensuremath{\flmwedgeq}"#, STIX), // ESTIMATES
    ('\u{225B}', r#"\ensuremath{\flmstareq}"#, STIX), // STAR EQUALS
    ('\u{225C}', r#"\ensuremath{\triangleq}"#, AMSSYMB), // DELTA EQUAL TO
    ('\u{2260}', r#"\ensuremath{\neq}"#, BUILTINS), // NOT EQUAL TO
    ('\u{2261}', r#"\ensuremath{\equiv}"#, BUILTINS), // IDENTICAL TO
    ('\u{2262}', r#"\ensuremath{\not\equiv}"#, BUILTINS), // NOT IDENTICAL TO
    ('\u{2264}', r#"\ensuremath{\leq}"#, BUILTINS), // LESS-THAN OR EQUAL TO
    ('\u{2265}', r#"\ensuremath{\geq}"#, BUILTINS), // GREATER-THAN OR EQUAL TO
    ('\u{2266}', r#"\ensuremath{\leqq}"#, AMSSYMB), // LESS-THAN OVER EQUAL TO
    ('\u{2267}', r#"\ensuremath{\geqq}"#, AMSSYMB), // GREATER-THAN OVER EQUAL TO
    ('\u{2268}', r#"\ensuremath{\lneqq}"#, AMSSYMB), // LESS-THAN BUT NOT EQUAL TO
    ('\u{2269}', r#"\ensuremath{\gneqq}"#, AMSSYMB), // GREATER-THAN BUT NOT EQUAL TO
    ('\u{226A}', r#"\ensuremath{\ll}"#, BUILTINS), // MUCH LESS-THAN
    ('\u{226B}', r#"\ensuremath{\gg}"#, BUILTINS), // MUCH GREATER-THAN
    ('\u{226C}', r#"\ensuremath{\between}"#, AMSSYMB), // BETWEEN
    ('\u{226D}', r#"\ensuremath{\not\kern-0.3em\times}"#, BUILTINS), // NOT EQUIVALENT TO
    ('\u{226E}', r#"\ensuremath{\nless}"#, AMSSYMB), // NOT LESS-THAN
    ('\u{226F}', r#"\ensuremath{\ngtr}"#, AMSSYMB), // NOT GREATER-THAN
    ('\u{2270}', r#"\ensuremath{\nleq}"#, AMSSYMB), // NEITHER LESS-THAN NOR EQUAL TO
    ('\u{2271}', r#"\ensuremath{\ngeq}"#, AMSSYMB), // NEITHER GREATER-THAN NOR EQUAL TO
    ('\u{2272}', r#"\ensuremath{\lesssim}"#, AMSSYMB), // LESS-THAN OR EQUIVALENT TO
    ('\u{2273}', r#"\ensuremath{\gtrsim}"#, AMSSYMB), // GREATER-THAN OR EQUIVALENT TO
    ('\u{2274}', r#"\ensuremath{\not\lesssim}"#, AMSSYMB), // NEITHER LESS-THAN NOR EQUIVALENT TO
    ('\u{2275}', r#"\ensuremath{\not\gtrsim}"#, AMSSYMB), // NEITHER GREATER-THAN NOR EQUIVALENT TO
    ('\u{2276}', r#"\ensuremath{\lessgtr}"#, AMSSYMB), // LESS-THAN OR GREATER-THAN
    ('\u{2277}', r#"\ensuremath{\gtrless}"#, AMSSYMB), // GREATER-THAN OR LESS-THAN
    ('\u{2278}', r#"\ensuremath{\flmnlessgtr}"#, STIX), // NEITHER LESS-THAN NOR GREATER-THAN
    ('\u{2279}', r#"\ensuremath{\flmngtrless}"#, STIX), // NEITHER GREATER-THAN NOR LESS-THAN
    ('\u{227A}', r#"\ensuremath{\prec}"#, BUILTINS), // PRECEDES
    ('\u{227B}', r#"\ensuremath{\succ}"#, BUILTINS), // SUCCEEDS
    ('\u{227C}', r#"\ensuremath{\preceq}"#, BUILTINS), // PRECEDES OR EQUAL TO
    ('\u{227D}', r#"\ensuremath{\succeq}"#, BUILTINS), // SUCCEEDS OR EQUAL TO
    ('\u{227E}', r#"\ensuremath{\precsim}"#, AMSSYMB), // PRECEDES OR EQUIVALENT TO
    ('\u{227F}', r#"\ensuremath{\succsim}"#, AMSSYMB), // SUCCEEDS OR EQUIVALENT TO
    ('\u{2280}', r#"\ensuremath{\nprec}"#, AMSSYMB), // DOES NOT PRECEDE
    ('\u{2281}', r#"\ensuremath{\nsucc}"#, AMSSYMB), // DOES NOT SUCCEED
    ('\u{2282}', r#"\ensuremath{\subset}"#, BUILTINS), // SUBSET OF
    ('\u{2283}', r#"\ensuremath{\supset}"#, BUILTINS), // SUPERSET OF
    ('\u{2284}', r#"\ensuremath{\not\subset}"#, BUILTINS), // NOT A SUBSET OF
    ('\u{2285}', r#"\ensuremath{\not\supset}"#, BUILTINS), // NOT A SUPERSET OF
    ('\u{2286}', r#"\ensuremath{\subseteq}"#, BUILTINS), // SUBSET OF OR EQUAL TO
    ('\u{2287}', r#"\ensuremath{\supseteq}"#, BUILTINS), // SUPERSET OF OR EQUAL TO
    ('\u{2288}', r#"\ensuremath{\nsubseteq}"#, AMSSYMB), // NEITHER A SUBSET OF NOR EQUAL TO
    ('\u{2289}', r#"\ensuremath{\nsupseteq}"#, AMSSYMB), // NEITHER A SUPERSET OF NOR EQUAL TO
    ('\u{228A}', r#"\ensuremath{\subsetneq}"#, AMSSYMB), // SUBSET OF WITH NOT EQUAL TO
    ('\u{228B}', r#"\ensuremath{\supsetneq}"#, AMSSYMB), // SUPERSET OF WITH NOT EQUAL TO
    ('\u{228E}', r#"\ensuremath{\uplus}"#, BUILTINS), // MULTISET UNION
    ('\u{228F}', r#"\ensuremath{\sqsubset}"#, AMSSYMB), // SQUARE IMAGE OF
    ('\u{2290}', r#"\ensuremath{\sqsupset}"#, AMSSYMB), // SQUARE ORIGINAL OF
    ('\u{2291}', r#"\ensuremath{\sqsubseteq}"#, BUILTINS), // SQUARE IMAGE OF OR EQUAL TO
    ('\u{2292}', r#"\ensuremath{\sqsupseteq}"#, BUILTINS), // SQUARE ORIGINAL OF OR EQUAL TO
    ('\u{2293}', r#"\ensuremath{\sqcap}"#, BUILTINS), // SQUARE CAP
    ('\u{2294}', r#"\ensuremath{\sqcup}"#, BUILTINS), // SQUARE CUP
    ('\u{2295}', r#"\ensuremath{\oplus}"#, BUILTINS), // CIRCLED PLUS
    ('\u{2296}', r#"\ensuremath{\ominus}"#, BUILTINS), // CIRCLED MINUS
    ('\u{2297}', r#"\ensuremath{\otimes}"#, BUILTINS), // CIRCLED TIMES
    ('\u{2298}', r#"\ensuremath{\oslash}"#, BUILTINS), // CIRCLED DIVISION SLASH
    ('\u{2299}', r#"\ensuremath{\odot}"#, BUILTINS), // CIRCLED DOT OPERATOR
    ('\u{229A}', r#"\ensuremath{\circledcirc}"#, AMSSYMB), // CIRCLED RING OPERATOR
    ('\u{229B}', r#"\ensuremath{\circledast}"#, AMSSYMB), // CIRCLED ASTERISK OPERATOR
    ('\u{229D}', r#"\ensuremath{\circleddash}"#, AMSSYMB), // CIRCLED DASH
    ('\u{229E}', r#"\ensuremath{\boxplus}"#, AMSSYMB), // SQUARED PLUS
    ('\u{229F}', r#"\ensuremath{\boxminus}"#, AMSSYMB), // SQUARED MINUS
    ('\u{22A0}', r#"\ensuremath{\boxtimes}"#, AMSSYMB), // SQUARED TIMES
    ('\u{22A1}', r#"\ensuremath{\boxdot}"#, AMSSYMB), // SQUARED DOT OPERATOR
    ('\u{22A2}', r#"\ensuremath{\vdash}"#, BUILTINS), // RIGHT TACK
    ('\u{22A3}', r#"\ensuremath{\dashv}"#, BUILTINS), // LEFT TACK
    ('\u{22A4}', r#"\ensuremath{\top}"#, BUILTINS), // DOWN TACK
    ('\u{22A5}', r#"\ensuremath{\perp}"#, BUILTINS), // UP TACK
    ('\u{22A7}', r#"\ensuremath{\flmmodels}"#, STIX), // MODELS
    ('\u{22A8}', r#"\ensuremath{\flmvDash}"#, STIX), // TRUE
    ('\u{22A9}', r#"\ensuremath{\Vdash}"#, AMSSYMB), // FORCES
    ('\u{22AA}', r#"\ensuremath{\Vvdash}"#, AMSSYMB), // TRIPLE VERTICAL BAR RIGHT TURNSTILE
    ('\u{22AB}', r#"\ensuremath{\flmVDash}"#, STIX), // DOUBLE VERTICAL BAR DOUBLE RIGHT TURNSTILE
    ('\u{22AC}', r#"\ensuremath{\nvdash}"#, AMSSYMB), // DOES NOT PROVE
    ('\u{22AD}', r#"\ensuremath{\nvDash}"#, AMSSYMB), // NOT TRUE
    ('\u{22AE}', r#"\ensuremath{\nVdash}"#, AMSSYMB), // DOES NOT FORCE
    ('\u{22AF}', r#"\ensuremath{\nVDash}"#, AMSSYMB), // NEGATED DOUBLE VERTICAL BAR DOUBLE RIGHT TURNSTILE
    ('\u{22B2}', r#"\ensuremath{\vartriangleleft}"#, AMSSYMB), // NORMAL SUBGROUP OF
    ('\u{22B3}', r#"\ensuremath{\vartriangleright}"#, AMSSYMB), // CONTAINS AS NORMAL SUBGROUP
    ('\u{22B4}', r#"\ensuremath{\trianglelefteq}"#, AMSSYMB), // NORMAL SUBGROUP OF OR EQUAL TO
    ('\u{22B5}', r#"\ensuremath{\trianglerighteq}"#, AMSSYMB), // CONTAINS AS NORMAL SUBGROUP OR EQUAL TO
    ('\u{22B6}', r#"\ensuremath{\flmorigof}"#, STIX), // ORIGINAL OF
    ('\u{22B7}', r#"\ensuremath{\flmimageof}"#, STIX), // IMAGE OF
    ('\u{22B8}', r#"\ensuremath{\multimap}"#, AMSSYMB), // MULTIMAP
    ('\u{22B9}', r#"\ensuremath{\flmhermitmatrix}"#, STIX), // HERMITIAN CONJUGATE MATRIX
    ('\u{22BA}', r#"\ensuremath{\intercal}"#, AMSSYMB), // INTERCALATE
    ('\u{22BB}', r#"\ensuremath{\veebar}"#, AMSSYMB), // XOR
    ('\u{22BE}', r#"\ensuremath{\flmmeasuredrightangle}"#, STIX), // RIGHT ANGLE WITH ARC
    ('\u{22C0}', r#"\ensuremath{\bigwedge}"#, BUILTINS), // N-ARY LOGICAL AND
    ('\u{22C1}', r#"\ensuremath{\bigvee}"#, BUILTINS), // N-ARY LOGICAL OR
    ('\u{22C2}', r#"\ensuremath{\bigcap}"#, BUILTINS), // N-ARY INTERSECTION
    ('\u{22C3}', r#"\ensuremath{\bigcup}"#, BUILTINS), // N-ARY UNION
    ('\u{22C4}', r#"\ensuremath{\diamond}"#, BUILTINS), // DIAMOND OPERATOR
    ('\u{22C5}', r#"\ensuremath{\cdot}"#, BUILTINS), // DOT OPERATOR
    ('\u{22C6}', r#"\ensuremath{\star}"#, BUILTINS), // STAR OPERATOR
    ('\u{22C7}', r#"\ensuremath{\divideontimes}"#, AMSSYMB), // DIVISION TIMES
    ('\u{22C8}', r#"\ensuremath{\bowtie}"#, BUILTINS), // BOWTIE
    ('\u{22C9}', r#"\ensuremath{\ltimes}"#, AMSSYMB), // LEFT NORMAL FACTOR SEMIDIRECT PRODUCT
    ('\u{22CA}', r#"\ensuremath{\rtimes}"#, AMSSYMB), // RIGHT NORMAL FACTOR SEMIDIRECT PRODUCT
    ('\u{22CB}', r#"\ensuremath{\leftthreetimes}"#, AMSSYMB), // LEFT SEMIDIRECT PRODUCT
    ('\u{22CC}', r#"\ensuremath{\rightthreetimes}"#, AMSSYMB), // RIGHT SEMIDIRECT PRODUCT
    ('\u{22CD}', r#"\ensuremath{\backsimeq}"#, AMSSYMB), // REVERSED TILDE EQUALS
    ('\u{22CE}', r#"\ensuremath{\curlyvee}"#, AMSSYMB), // CURLY LOGICAL OR
    ('\u{22CF}', r#"\ensuremath{\curlywedge}"#, AMSSYMB), // CURLY LOGICAL AND
    ('\u{22D0}', r#"\ensuremath{\Subset}"#, AMSSYMB), // DOUBLE SUBSET
    ('\u{22D1}', r#"\ensuremath{\Supset}"#, AMSSYMB), // DOUBLE SUPERSET
    ('\u{22D2}', r#"\ensuremath{\Cap}"#, AMSSYMB), // DOUBLE INTERSECTION
    ('\u{22D3}', r#"\ensuremath{\Cup}"#, AMSSYMB), // DOUBLE UNION
    ('\u{22D4}', r#"\ensuremath{\pitchfork}"#, AMSSYMB), // PITCHFORK
    ('\u{22D6}', r#"\ensuremath{\lessdot}"#, AMSSYMB), // LESS-THAN WITH DOT
    ('\u{22D7}', r#"\ensuremath{\gtrdot}"#, AMSSYMB), // GREATER-THAN WITH DOT
    ('\u{22D8}', r#"\ensuremath{\flmlll}"#, STIX), // VERY MUCH LESS-THAN
    ('\u{22D9}', r#"\ensuremath{\flmggg}"#, STIX), // VERY MUCH GREATER-THAN
    ('\u{22DA}', r#"\ensuremath{\lesseqgtr}"#, AMSSYMB), // LESS-THAN EQUAL TO OR GREATER-THAN
    ('\u{22DB}', r#"\ensuremath{\gtreqless}"#, AMSSYMB), // GREATER-THAN EQUAL TO OR LESS-THAN
    ('\u{22DE}', r#"\ensuremath{\curlyeqprec}"#, AMSSYMB), // EQUAL TO OR PRECEDES
    ('\u{22DF}', r#"\ensuremath{\curlyeqsucc}"#, AMSSYMB), // EQUAL TO OR SUCCEEDS
    ('\u{22E2}', r#"\ensuremath{\not\sqsubseteq}"#, BUILTINS), // NOT SQUARE IMAGE OF OR EQUAL TO
    ('\u{22E3}', r#"\ensuremath{\not\sqsupseteq}"#, BUILTINS), // NOT SQUARE ORIGINAL OF OR EQUAL TO
    ('\u{22E6}', r#"\ensuremath{\lnsim}"#, AMSSYMB), // LESS-THAN BUT NOT EQUIVALENT TO
    ('\u{22E7}', r#"\ensuremath{\gnsim}"#, AMSSYMB), // GREATER-THAN BUT NOT EQUIVALENT TO
    ('\u{22E8}', r#"\ensuremath{\flmprecnsim}"#, STIX), // PRECEDES BUT NOT EQUIVALENT TO
    ('\u{22E9}', r#"\ensuremath{\succnsim}"#, AMSSYMB), // SUCCEEDS BUT NOT EQUIVALENT TO
    ('\u{22EA}', r#"\ensuremath{\ntriangleleft}"#, AMSSYMB), // NOT NORMAL SUBGROUP OF
    ('\u{22EB}', r#"\ensuremath{\ntriangleright}"#, AMSSYMB), // DOES NOT CONTAIN AS NORMAL SUBGROUP
    ('\u{22EC}', r#"\ensuremath{\ntrianglelefteq}"#, AMSSYMB), // NOT NORMAL SUBGROUP OF OR EQUAL TO
    ('\u{22ED}', r#"\ensuremath{\ntrianglerighteq}"#, AMSSYMB), // DOES NOT CONTAIN AS NORMAL SUBGROUP OR EQUAL
    ('\u{22EE}', r#"\ensuremath{\vdots}"#, BUILTINS), // VERTICAL ELLIPSIS
    ('\u{22EF}', r#"\ensuremath{\cdots}"#, BUILTINS), // MIDLINE HORIZONTAL ELLIPSIS
    ('\u{22F0}', r#"\ensuremath{\flmadots}"#, STIX), // UP RIGHT DIAGONAL ELLIPSIS
    ('\u{22F1}', r#"\ensuremath{\ddots}"#, BUILTINS), // DOWN RIGHT DIAGONAL ELLIPSIS
    ('\u{2305}', r#"\ensuremath{\barwedge}"#, AMSSYMB), // PROJECTIVE
    ('\u{2306}', r#"\ensuremath{\flmvardoublebarwedge}"#, STIX), // PERSPECTIVE
    ('\u{2308}', r#"\ensuremath{\lceil}"#, BUILTINS), // LEFT CEILING
    ('\u{2309}', r#"\ensuremath{\rceil}"#, BUILTINS), // RIGHT CEILING
    ('\u{230A}', r#"\ensuremath{\lfloor}"#, BUILTINS), // LEFT FLOOR
    ('\u{230B}', r#"\ensuremath{\rfloor}"#, BUILTINS), // RIGHT FLOOR
    ('\u{2315}', r#"\ensuremath{\flmrecorder}"#, WASY), // TELEPHONE RECORDER
    ('\u{2316}', r#"\ensuremath{\mathchar"2208}"#, BUILTINS), // POSITION INDICATOR
    ('\u{231C}', r#"\ensuremath{\ulcorner}"#, AMSSYMB), // TOP LEFT CORNER
    ('\u{231D}', r#"\ensuremath{\urcorner}"#, AMSSYMB), // TOP RIGHT CORNER
    ('\u{231E}', r#"\ensuremath{\llcorner}"#, AMSSYMB), // BOTTOM LEFT CORNER
    ('\u{231F}', r#"\ensuremath{\lrcorner}"#, AMSSYMB), // BOTTOM RIGHT CORNER
    ('\u{2322}', r#"\ensuremath{\frown}"#, BUILTINS), // FROWN
    ('\u{2323}', r#"\ensuremath{\smile}"#, BUILTINS), // SMILE
    ('\u{2329}', r#"\textlangle"#, BUILTINS), // LEFT-POINTING ANGLE BRACKET
    ('\u{232A}', r#"\textrangle"#, BUILTINS), // RIGHT-POINTING ANGLE BRACKET
    ('\u{23B0}', r#"\ensuremath{\lmoustache}"#, BUILTINS), // UPPER LEFT OR LOWER RIGHT CURLY BRACKET SECTION
    ('\u{23B1}', r#"\ensuremath{\rmoustache}"#, BUILTINS), // UPPER RIGHT OR LOWER LEFT CURLY BRACKET SECTION
    ('\u{2422}', r#"\textblank"#, BUILTINS), // BLANK SYMBOL
    ('\u{2423}', r#"\textvisiblespace"#, BUILTINS), // OPEN BOX
    ('\u{25A0}', r#"\ensuremath{\blacksquare}"#, AMSSYMB), // BLACK SQUARE
    ('\u{25A1}', r#"\ensuremath{\square}"#, AMSSYMB), // WHITE SQUARE
    ('\u{25AA}', r#"{\small\ensuremath{\blacksquare}}"#, AMSSYMB), // BLACK SMALL SQUARE
    ('\u{25AD}', r#"\fbox{~~}"#, BUILTINS), // WHITE RECTANGLE
    ('\u{25B3}', r#"\ensuremath{\bigtriangleup}"#, BUILTINS), // WHITE UP-POINTING TRIANGLE
    ('\u{25B4}', r#"\ensuremath{\blacktriangle}"#, AMSSYMB), // BLACK UP-POINTING SMALL TRIANGLE
    ('\u{25B5}', r#"\ensuremath{\vartriangle}"#, AMSSYMB), // WHITE UP-POINTING SMALL TRIANGLE
    ('\u{25B8}', r#"\ensuremath{\blacktriangleright}"#, AMSSYMB), // BLACK RIGHT-POINTING SMALL TRIANGLE
    ('\u{25B9}', r#"\ensuremath{\triangleright}"#, BUILTINS), // WHITE RIGHT-POINTING SMALL TRIANGLE
    ('\u{25BD}', r#"\ensuremath{\bigtriangledown}"#, BUILTINS), // WHITE DOWN-POINTING TRIANGLE
    ('\u{25BE}', r#"\ensuremath{\blacktriangledown}"#, AMSSYMB), // BLACK DOWN-POINTING SMALL TRIANGLE
    ('\u{25BF}', r#"\ensuremath{\triangledown}"#, AMSSYMB), // WHITE DOWN-POINTING SMALL TRIANGLE
    ('\u{25C2}', r#"\ensuremath{\blacktriangleleft}"#, AMSSYMB), // BLACK LEFT-POINTING SMALL TRIANGLE
    ('\u{25C3}', r#"\ensuremath{\triangleleft}"#, BUILTINS), // WHITE LEFT-POINTING SMALL TRIANGLE
    ('\u{25CA}', r#"\ensuremath{\lozenge}"#, AMSSYMB), // LOZENGE
    ('\u{25CB}', r#"\ensuremath{\bigcirc}"#, BUILTINS), // WHITE CIRCLE
    ('\u{25E6}', r#"\textopenbullet"#, BUILTINS), // WHITE BULLET
    ('\u{25EF}', r#"\textbigcircle"#, BUILTINS), // LARGE CIRCLE
    ('\u{2662}', r#"\ensuremath{\diamond}"#, BUILTINS), // WHITE DIAMOND SUIT
    ('\u{2669}', r#"\ensuremath{\flmquarternote}"#, STIX), // QUARTER NOTE
    ('\u{266A}', r#"\textmusicalnote"#, BUILTINS), // EIGHTH NOTE
    ('\u{266D}', r#"\ensuremath{\flat}"#, BUILTINS), // MUSIC FLAT SIGN
    ('\u{266E}', r#"\ensuremath{\natural}"#, BUILTINS), // MUSIC NATURAL SIGN
    ('\u{266F}', r#"\ensuremath{\sharp}"#, BUILTINS), // MUSIC SHARP SIGN
    ('\u{27E8}', r#"\ensuremath{\langle}"#, BUILTINS), // MATHEMATICAL LEFT ANGLE BRACKET
    ('\u{27E9}', r#"\ensuremath{\rangle}"#, BUILTINS), // MATHEMATICAL RIGHT ANGLE BRACKET
    ('\u{27F5}', r#"\ensuremath{\longleftarrow}"#, BUILTINS), // LONG LEFTWARDS ARROW
    ('\u{27F6}', r#"\ensuremath{\longrightarrow}"#, BUILTINS), // LONG RIGHTWARDS ARROW
    ('\u{27F7}', r#"\ensuremath{\longleftrightarrow}"#, BUILTINS), // LONG LEFT RIGHT ARROW
    ('\u{27F8}', r#"\ensuremath{\Longleftarrow}"#, BUILTINS), // LONG LEFTWARDS DOUBLE ARROW
    ('\u{27F9}', r#"\ensuremath{\Longrightarrow}"#, BUILTINS), // LONG RIGHTWARDS DOUBLE ARROW
    ('\u{27FA}', r#"\ensuremath{\Longleftrightarrow}"#, BUILTINS), // LONG LEFT RIGHT DOUBLE ARROW
    ('\u{27FC}', r#"\ensuremath{\longmapsto}"#, BUILTINS), // LONG RIGHTWARDS ARROW FROM BAR
    ('\u{27FF}', r#"\ensuremath{\sim\joinrel\leadsto}"#, AMSSYMB), // LONG RIGHTWARDS SQUIGGLE ARROW
    ('\u{2993}', r#"\ensuremath{<\kern-0.58em(}"#, BUILTINS), // LEFT ARC LESS-THAN BRACKET
    ('\u{29EB}', r#"\ensuremath{\blacklozenge}"#, AMSSYMB), // BLACK LOZENGE
    ('\u{2A0F}', r#"\ensuremath{\flmfint}"#, STIX), // INTEGRAL AVERAGE WITH SLASH
    ('\u{2A16}', r#"\ensuremath{\flmsqint}"#, STIX), // QUATERNION INTEGRAL OPERATOR
    ('\u{2A3F}', r#"\ensuremath{\amalg}"#, BUILTINS), // AMALGAMATION OR COPRODUCT
    ('\u{2A6E}', r#"\ensuremath{\stackrel{*}{=}}"#, BUILTINS), // EQUALS WITH ASTERISK
    ('\u{2A75}', r#"=="#, BUILTINS), // TWO CONSECUTIVE EQUALS SIGNS
    ('\u{2A7D}', r#"\ensuremath{\leqslant}"#, AMSSYMB), // LESS-THAN OR SLANTED EQUAL TO
    ('\u{2A7E}', r#"\ensuremath{\geqslant}"#, AMSSYMB), // GREATER-THAN OR SLANTED EQUAL TO
    ('\u{2A85}', r#"\ensuremath{\lessapprox}"#, AMSSYMB), // LESS-THAN OR APPROXIMATE
    ('\u{2A86}', r#"\ensuremath{\gtrapprox}"#, AMSSYMB), // GREATER-THAN OR APPROXIMATE
    ('\u{2A87}', r#"\ensuremath{\lneq}"#, AMSSYMB), // LESS-THAN AND SINGLE-LINE NOT EQUAL TO
    ('\u{2A88}', r#"\ensuremath{\gneq}"#, AMSSYMB), // GREATER-THAN AND SINGLE-LINE NOT EQUAL TO
    ('\u{2A89}', r#"\ensuremath{\lnapprox}"#, AMSSYMB), // LESS-THAN AND NOT APPROXIMATE
    ('\u{2A8A}', r#"\ensuremath{\gnapprox}"#, AMSSYMB), // GREATER-THAN AND NOT APPROXIMATE
    ('\u{2A8B}', r#"\ensuremath{\lesseqqgtr}"#, AMSSYMB), // LESS-THAN ABOVE DOUBLE-LINE EQUAL ABOVE GREATER-THAN
    ('\u{2A8C}', r#"\ensuremath{\gtreqqless}"#, AMSSYMB), // GREATER-THAN ABOVE DOUBLE-LINE EQUAL ABOVE LESS-THAN
    ('\u{2A95}', r#"\ensuremath{\eqslantless}"#, AMSSYMB), // SLANTED EQUAL TO OR LESS-THAN
    ('\u{2A96}', r#"\ensuremath{\eqslantgtr}"#, AMSSYMB), // SLANTED EQUAL TO OR GREATER-THAN
    ('\u{2AAF}', r#"\ensuremath{\preceq}"#, BUILTINS), // PRECEDES ABOVE SINGLE-LINE EQUALS SIGN
    ('\u{2AB0}', r#"\ensuremath{\succeq}"#, BUILTINS), // SUCCEEDS ABOVE SINGLE-LINE EQUALS SIGN
    ('\u{2AB5}', r#"\ensuremath{\precneqq}"#, AMSSYMB), // PRECEDES ABOVE NOT EQUAL TO
    ('\u{2AB6}', r#"\ensuremath{\succneqq}"#, AMSSYMB), // SUCCEEDS ABOVE NOT EQUAL TO
    ('\u{2AB7}', r#"\ensuremath{\precapprox}"#, AMSSYMB), // PRECEDES ABOVE ALMOST EQUAL TO
    ('\u{2AB8}', r#"\ensuremath{\succapprox}"#, AMSSYMB), // SUCCEEDS ABOVE ALMOST EQUAL TO
    ('\u{2AB9}', r#"\ensuremath{\precnapprox}"#, AMSSYMB), // PRECEDES ABOVE NOT ALMOST EQUAL TO
    ('\u{2ABA}', r#"\ensuremath{\succnapprox}"#, AMSSYMB), // SUCCEEDS ABOVE NOT ALMOST EQUAL TO
    ('\u{2AC5}', r#"\ensuremath{\subseteqq}"#, AMSSYMB), // SUBSET OF ABOVE EQUALS SIGN
    ('\u{2AC6}', r#"\ensuremath{\supseteqq}"#, AMSSYMB), // SUPERSET OF ABOVE EQUALS SIGN
    ('\u{2ACB}', r#"\ensuremath{\subsetneqq}"#, AMSSYMB), // SUBSET OF ABOVE NOT EQUAL TO
    ('\u{2ACC}', r#"\ensuremath{\supsetneqq}"#, AMSSYMB), // SUPERSET OF ABOVE NOT EQUAL TO
    ('\u{2AFD}', r#"\ensuremath{{{/}\!\!{/}}}"#, BUILTINS), // DOUBLE SOLIDUS OPERATOR
    ('\u{3008}', r#"\ensuremath{\langle}"#, BUILTINS), // LEFT ANGLE BRACKET
    ('\u{3009}', r#"\ensuremath{\rangle}"#, BUILTINS), // RIGHT ANGLE BRACKET
    ('\u{FB00}', r#"ff"#, BUILTINS), // LATIN SMALL LIGATURE FF
    ('\u{FB01}', r#"fi"#, BUILTINS), // LATIN SMALL LIGATURE FI
    ('\u{FB02}', r#"fl"#, BUILTINS), // LATIN SMALL LIGATURE FL
    ('\u{FB03}', r#"ffi"#, BUILTINS), // LATIN SMALL LIGATURE FFI
    ('\u{FB04}', r#"ffl"#, BUILTINS), // LATIN SMALL LIGATURE FFL
    ('\u{1D400}', r#"\ensuremath{\mathbf{A}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL A
    ('\u{1D401}', r#"\ensuremath{\mathbf{B}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL B
    ('\u{1D402}', r#"\ensuremath{\mathbf{C}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL C
    ('\u{1D403}', r#"\ensuremath{\mathbf{D}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL D
    ('\u{1D404}', r#"\ensuremath{\mathbf{E}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL E
    ('\u{1D405}', r#"\ensuremath{\mathbf{F}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL F
    ('\u{1D406}', r#"\ensuremath{\mathbf{G}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL G
    ('\u{1D407}', r#"\ensuremath{\mathbf{H}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL H
    ('\u{1D408}', r#"\ensuremath{\mathbf{I}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL I
    ('\u{1D409}', r#"\ensuremath{\mathbf{J}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL J
    ('\u{1D40A}', r#"\ensuremath{\mathbf{K}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL K
    ('\u{1D40B}', r#"\ensuremath{\mathbf{L}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL L
    ('\u{1D40C}', r#"\ensuremath{\mathbf{M}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL M
    ('\u{1D40D}', r#"\ensuremath{\mathbf{N}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL N
    ('\u{1D40E}', r#"\ensuremath{\mathbf{O}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL O
    ('\u{1D40F}', r#"\ensuremath{\mathbf{P}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL P
    ('\u{1D410}', r#"\ensuremath{\mathbf{Q}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL Q
    ('\u{1D411}', r#"\ensuremath{\mathbf{R}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL R
    ('\u{1D412}', r#"\ensuremath{\mathbf{S}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL S
    ('\u{1D413}', r#"\ensuremath{\mathbf{T}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL T
    ('\u{1D414}', r#"\ensuremath{\mathbf{U}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL U
    ('\u{1D415}', r#"\ensuremath{\mathbf{V}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL V
    ('\u{1D416}', r#"\ensuremath{\mathbf{W}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL W
    ('\u{1D417}', r#"\ensuremath{\mathbf{X}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL X
    ('\u{1D418}', r#"\ensuremath{\mathbf{Y}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL Y
    ('\u{1D419}', r#"\ensuremath{\mathbf{Z}}"#, BUILTINS), // MATHEMATICAL BOLD CAPITAL Z
    ('\u{1D41A}', r#"\ensuremath{\mathbf{a}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL A
    ('\u{1D41B}', r#"\ensuremath{\mathbf{b}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL B
    ('\u{1D41C}', r#"\ensuremath{\mathbf{c}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL C
    ('\u{1D41D}', r#"\ensuremath{\mathbf{d}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL D
    ('\u{1D41E}', r#"\ensuremath{\mathbf{e}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL E
    ('\u{1D41F}', r#"\ensuremath{\mathbf{f}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL F
    ('\u{1D420}', r#"\ensuremath{\mathbf{g}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL G
    ('\u{1D421}', r#"\ensuremath{\mathbf{h}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL H
    ('\u{1D422}', r#"\ensuremath{\mathbf{i}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL I
    ('\u{1D423}', r#"\ensuremath{\mathbf{j}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL J
    ('\u{1D424}', r#"\ensuremath{\mathbf{k}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL K
    ('\u{1D425}', r#"\ensuremath{\mathbf{l}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL L
    ('\u{1D426}', r#"\ensuremath{\mathbf{m}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL M
    ('\u{1D427}', r#"\ensuremath{\mathbf{n}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL N
    ('\u{1D428}', r#"\ensuremath{\mathbf{o}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL O
    ('\u{1D429}', r#"\ensuremath{\mathbf{p}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL P
    ('\u{1D42A}', r#"\ensuremath{\mathbf{q}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL Q
    ('\u{1D42B}', r#"\ensuremath{\mathbf{r}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL R
    ('\u{1D42C}', r#"\ensuremath{\mathbf{s}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL S
    ('\u{1D42D}', r#"\ensuremath{\mathbf{t}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL T
    ('\u{1D42E}', r#"\ensuremath{\mathbf{u}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL U
    ('\u{1D42F}', r#"\ensuremath{\mathbf{v}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL V
    ('\u{1D430}', r#"\ensuremath{\mathbf{w}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL W
    ('\u{1D431}', r#"\ensuremath{\mathbf{x}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL X
    ('\u{1D432}', r#"\ensuremath{\mathbf{y}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL Y
    ('\u{1D433}', r#"\ensuremath{\mathbf{z}}"#, BUILTINS), // MATHEMATICAL BOLD SMALL Z
    ('\u{1D434}', r#"\ensuremath{\mathit{A}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL A
    ('\u{1D435}', r#"\ensuremath{\mathit{B}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL B
    ('\u{1D436}', r#"\ensuremath{\mathit{C}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL C
    ('\u{1D437}', r#"\ensuremath{\mathit{D}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL D
    ('\u{1D438}', r#"\ensuremath{\mathit{E}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL E
    ('\u{1D439}', r#"\ensuremath{\mathit{F}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL F
    ('\u{1D43A}', r#"\ensuremath{\mathit{G}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL G
    ('\u{1D43B}', r#"\ensuremath{\mathit{H}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL H
    ('\u{1D43C}', r#"\ensuremath{\mathit{I}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL I
    ('\u{1D43D}', r#"\ensuremath{\mathit{J}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL J
    ('\u{1D43E}', r#"\ensuremath{\mathit{K}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL K
    ('\u{1D43F}', r#"\ensuremath{\mathit{L}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL L
    ('\u{1D440}', r#"\ensuremath{\mathit{M}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL M
    ('\u{1D441}', r#"\ensuremath{\mathit{N}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL N
    ('\u{1D442}', r#"\ensuremath{\mathit{O}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL O
    ('\u{1D443}', r#"\ensuremath{\mathit{P}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL P
    ('\u{1D444}', r#"\ensuremath{\mathit{Q}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL Q
    ('\u{1D445}', r#"\ensuremath{\mathit{R}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL R
    ('\u{1D446}', r#"\ensuremath{\mathit{S}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL S
    ('\u{1D447}', r#"\ensuremath{\mathit{T}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL T
    ('\u{1D448}', r#"\ensuremath{\mathit{U}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL U
    ('\u{1D449}', r#"\ensuremath{\mathit{V}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL V
    ('\u{1D44A}', r#"\ensuremath{\mathit{W}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL W
    ('\u{1D44B}', r#"\ensuremath{\mathit{X}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL X
    ('\u{1D44C}', r#"\ensuremath{\mathit{Y}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL Y
    ('\u{1D44D}', r#"\ensuremath{\mathit{Z}}"#, BUILTINS), // MATHEMATICAL ITALIC CAPITAL Z
    ('\u{1D44E}', r#"\ensuremath{\mathit{a}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL A
    ('\u{1D44F}', r#"\ensuremath{\mathit{b}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL B
    ('\u{1D450}', r#"\ensuremath{\mathit{c}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL C
    ('\u{1D451}', r#"\ensuremath{\mathit{d}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL D
    ('\u{1D452}', r#"\ensuremath{\mathit{e}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL E
    ('\u{1D453}', r#"\ensuremath{\mathit{f}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL F
    ('\u{1D454}', r#"\ensuremath{\mathit{g}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL G
    ('\u{1D455}', r#"\ensuremath{\mathit{h}}"#, BUILTINS), // (unnamed code point)
    ('\u{1D456}', r#"\ensuremath{\mathit{i}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL I
    ('\u{1D457}', r#"\ensuremath{\mathit{j}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL J
    ('\u{1D458}', r#"\ensuremath{\mathit{k}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL K
    ('\u{1D459}', r#"\ensuremath{\mathit{l}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL L
    ('\u{1D45A}', r#"\ensuremath{\mathit{m}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL M
    ('\u{1D45B}', r#"\ensuremath{\mathit{n}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL N
    ('\u{1D45C}', r#"\ensuremath{\mathit{o}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL O
    ('\u{1D45D}', r#"\ensuremath{\mathit{p}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL P
    ('\u{1D45E}', r#"\ensuremath{\mathit{q}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL Q
    ('\u{1D45F}', r#"\ensuremath{\mathit{r}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL R
    ('\u{1D460}', r#"\ensuremath{\mathit{s}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL S
    ('\u{1D461}', r#"\ensuremath{\mathit{t}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL T
    ('\u{1D462}', r#"\ensuremath{\mathit{u}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL U
    ('\u{1D463}', r#"\ensuremath{\mathit{v}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL V
    ('\u{1D464}', r#"\ensuremath{\mathit{w}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL W
    ('\u{1D465}', r#"\ensuremath{\mathit{x}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL X
    ('\u{1D466}', r#"\ensuremath{\mathit{y}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL Y
    ('\u{1D467}', r#"\ensuremath{\mathit{z}}"#, BUILTINS), // MATHEMATICAL ITALIC SMALL Z
    ('\u{1D468}', r#"\ensuremath{\boldsymbol{\mathit{A}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL A
    ('\u{1D469}', r#"\ensuremath{\boldsymbol{\mathit{B}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL B
    ('\u{1D46A}', r#"\ensuremath{\boldsymbol{\mathit{C}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL C
    ('\u{1D46B}', r#"\ensuremath{\boldsymbol{\mathit{D}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL D
    ('\u{1D46C}', r#"\ensuremath{\boldsymbol{\mathit{E}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL E
    ('\u{1D46D}', r#"\ensuremath{\boldsymbol{\mathit{F}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL F
    ('\u{1D46E}', r#"\ensuremath{\boldsymbol{\mathit{G}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL G
    ('\u{1D46F}', r#"\ensuremath{\boldsymbol{\mathit{H}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL H
    ('\u{1D470}', r#"\ensuremath{\boldsymbol{\mathit{I}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL I
    ('\u{1D471}', r#"\ensuremath{\boldsymbol{\mathit{J}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL J
    ('\u{1D472}', r#"\ensuremath{\boldsymbol{\mathit{K}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL K
    ('\u{1D473}', r#"\ensuremath{\boldsymbol{\mathit{L}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL L
    ('\u{1D474}', r#"\ensuremath{\boldsymbol{\mathit{M}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL M
    ('\u{1D475}', r#"\ensuremath{\boldsymbol{\mathit{N}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL N
    ('\u{1D476}', r#"\ensuremath{\boldsymbol{\mathit{O}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL O
    ('\u{1D477}', r#"\ensuremath{\boldsymbol{\mathit{P}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL P
    ('\u{1D478}', r#"\ensuremath{\boldsymbol{\mathit{Q}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL Q
    ('\u{1D479}', r#"\ensuremath{\boldsymbol{\mathit{R}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL R
    ('\u{1D47A}', r#"\ensuremath{\boldsymbol{\mathit{S}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL S
    ('\u{1D47B}', r#"\ensuremath{\boldsymbol{\mathit{T}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL T
    ('\u{1D47C}', r#"\ensuremath{\boldsymbol{\mathit{U}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL U
    ('\u{1D47D}', r#"\ensuremath{\boldsymbol{\mathit{V}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL V
    ('\u{1D47E}', r#"\ensuremath{\boldsymbol{\mathit{W}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL W
    ('\u{1D47F}', r#"\ensuremath{\boldsymbol{\mathit{X}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL X
    ('\u{1D480}', r#"\ensuremath{\boldsymbol{\mathit{Y}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL Y
    ('\u{1D481}', r#"\ensuremath{\boldsymbol{\mathit{Z}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC CAPITAL Z
    ('\u{1D482}', r#"\ensuremath{\boldsymbol{\mathit{a}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL A
    ('\u{1D483}', r#"\ensuremath{\boldsymbol{\mathit{b}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL B
    ('\u{1D484}', r#"\ensuremath{\boldsymbol{\mathit{c}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL C
    ('\u{1D485}', r#"\ensuremath{\boldsymbol{\mathit{d}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL D
    ('\u{1D486}', r#"\ensuremath{\boldsymbol{\mathit{e}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL E
    ('\u{1D487}', r#"\ensuremath{\boldsymbol{\mathit{f}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL F
    ('\u{1D488}', r#"\ensuremath{\boldsymbol{\mathit{g}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL G
    ('\u{1D489}', r#"\ensuremath{\boldsymbol{\mathit{h}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL H
    ('\u{1D48A}', r#"\ensuremath{\boldsymbol{\mathit{i}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL I
    ('\u{1D48B}', r#"\ensuremath{\boldsymbol{\mathit{j}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL J
    ('\u{1D48C}', r#"\ensuremath{\boldsymbol{\mathit{k}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL K
    ('\u{1D48D}', r#"\ensuremath{\boldsymbol{\mathit{l}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL L
    ('\u{1D48E}', r#"\ensuremath{\boldsymbol{\mathit{m}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL M
    ('\u{1D48F}', r#"\ensuremath{\boldsymbol{\mathit{n}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL N
    ('\u{1D490}', r#"\ensuremath{\boldsymbol{\mathit{o}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL O
    ('\u{1D491}', r#"\ensuremath{\boldsymbol{\mathit{p}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL P
    ('\u{1D492}', r#"\ensuremath{\boldsymbol{\mathit{q}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL Q
    ('\u{1D493}', r#"\ensuremath{\boldsymbol{\mathit{r}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL R
    ('\u{1D494}', r#"\ensuremath{\boldsymbol{\mathit{s}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL S
    ('\u{1D495}', r#"\ensuremath{\boldsymbol{\mathit{t}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL T
    ('\u{1D496}', r#"\ensuremath{\boldsymbol{\mathit{u}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL U
    ('\u{1D497}', r#"\ensuremath{\boldsymbol{\mathit{v}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL V
    ('\u{1D498}', r#"\ensuremath{\boldsymbol{\mathit{w}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL W
    ('\u{1D499}', r#"\ensuremath{\boldsymbol{\mathit{x}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL X
    ('\u{1D49A}', r#"\ensuremath{\boldsymbol{\mathit{y}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL Y
    ('\u{1D49B}', r#"\ensuremath{\boldsymbol{\mathit{z}}}"#, AMSMATH), // MATHEMATICAL BOLD ITALIC SMALL Z
    ('\u{1D49C}', r#"\ensuremath{\mathscr{A}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL A
    ('\u{1D49D}', r#"\ensuremath{\mathscr{B}}"#, MATHRSFS), // (unnamed code point)
    ('\u{1D49E}', r#"\ensuremath{\mathscr{C}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL C
    ('\u{1D49F}', r#"\ensuremath{\mathscr{D}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL D
    ('\u{1D4A0}', r#"\ensuremath{\mathscr{E}}"#, MATHRSFS), // (unnamed code point)
    ('\u{1D4A1}', r#"\ensuremath{\mathscr{F}}"#, MATHRSFS), // (unnamed code point)
    ('\u{1D4A2}', r#"\ensuremath{\mathscr{G}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL G
    ('\u{1D4A3}', r#"\ensuremath{\mathscr{H}}"#, MATHRSFS), // (unnamed code point)
    ('\u{1D4A4}', r#"\ensuremath{\mathscr{I}}"#, MATHRSFS), // (unnamed code point)
    ('\u{1D4A5}', r#"\ensuremath{\mathscr{J}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL J
    ('\u{1D4A6}', r#"\ensuremath{\mathscr{K}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL K
    ('\u{1D4A7}', r#"\ensuremath{\mathscr{L}}"#, MATHRSFS), // (unnamed code point)
    ('\u{1D4A8}', r#"\ensuremath{\mathscr{M}}"#, MATHRSFS), // (unnamed code point)
    ('\u{1D4A9}', r#"\ensuremath{\mathscr{N}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL N
    ('\u{1D4AA}', r#"\ensuremath{\mathscr{O}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL O
    ('\u{1D4AB}', r#"\ensuremath{\mathscr{P}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL P
    ('\u{1D4AC}', r#"\ensuremath{\mathscr{Q}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL Q
    ('\u{1D4AD}', r#"\ensuremath{\mathscr{R}}"#, MATHRSFS), // (unnamed code point)
    ('\u{1D4AE}', r#"\ensuremath{\mathscr{S}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL S
    ('\u{1D4AF}', r#"\ensuremath{\mathscr{T}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL T
    ('\u{1D4B0}', r#"\ensuremath{\mathscr{U}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL U
    ('\u{1D4B1}', r#"\ensuremath{\mathscr{V}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL V
    ('\u{1D4B2}', r#"\ensuremath{\mathscr{W}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL W
    ('\u{1D4B3}', r#"\ensuremath{\mathscr{X}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL X
    ('\u{1D4B4}', r#"\ensuremath{\mathscr{Y}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL Y
    ('\u{1D4B5}', r#"\ensuremath{\mathscr{Z}}"#, MATHRSFS), // MATHEMATICAL SCRIPT CAPITAL Z
    ('\u{1D4B6}', r#"\ensuremath{\flmScr{a}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL A
    ('\u{1D4B7}', r#"\ensuremath{\flmScr{b}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL B
    ('\u{1D4B8}', r#"\ensuremath{\flmScr{c}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL C
    ('\u{1D4B9}', r#"\ensuremath{\flmScr{d}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL D
    ('\u{1D4BB}', r#"\ensuremath{\flmScr{f}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL F
    ('\u{1D4BD}', r#"\ensuremath{\flmScr{h}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL H
    ('\u{1D4BE}', r#"\ensuremath{\flmScr{i}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL I
    ('\u{1D4BF}', r#"\ensuremath{\flmScr{j}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL J
    ('\u{1D4C0}', r#"\ensuremath{\flmScr{k}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL K
    ('\u{1D4C1}', r#"\ensuremath{\flmScr{l}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL L
    ('\u{1D4C2}', r#"\ensuremath{\flmScr{m}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL M
    ('\u{1D4C3}', r#"\ensuremath{\flmScr{n}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL N
    ('\u{1D4C5}', r#"\ensuremath{\flmScr{p}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL P
    ('\u{1D4C6}', r#"\ensuremath{\flmScr{q}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL Q
    ('\u{1D4C7}', r#"\ensuremath{\flmScr{r}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL R
    ('\u{1D4C8}', r#"\ensuremath{\flmScr{s}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL S
    ('\u{1D4C9}', r#"\ensuremath{\flmScr{t}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL T
    ('\u{1D4CA}', r#"\ensuremath{\flmScr{u}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL U
    ('\u{1D4CB}', r#"\ensuremath{\flmScr{v}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL V
    ('\u{1D4CC}', r#"\ensuremath{\flmScr{w}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL W
    ('\u{1D4CD}', r#"\ensuremath{\flmScr{x}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL X
    ('\u{1D4CE}', r#"\ensuremath{\flmScr{y}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL Y
    ('\u{1D4CF}', r#"\ensuremath{\flmScr{z}}"#, SCRIPT), // MATHEMATICAL SCRIPT SMALL Z
    ('\u{1D504}', r#"\ensuremath{\mathfrak{A}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL A
    ('\u{1D505}', r#"\ensuremath{\mathfrak{B}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL B
    ('\u{1D506}', r#"\ensuremath{\mathfrak{C}}"#, AMSSYMB), // (unnamed code point)
    ('\u{1D507}', r#"\ensuremath{\mathfrak{D}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL D
    ('\u{1D508}', r#"\ensuremath{\mathfrak{E}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL E
    ('\u{1D509}', r#"\ensuremath{\mathfrak{F}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL F
    ('\u{1D50A}', r#"\ensuremath{\mathfrak{G}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL G
    ('\u{1D50B}', r#"\ensuremath{\mathfrak{H}}"#, AMSSYMB), // (unnamed code point)
    ('\u{1D50C}', r#"\ensuremath{\mathfrak{I}}"#, AMSSYMB), // (unnamed code point)
    ('\u{1D50D}', r#"\ensuremath{\mathfrak{J}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL J
    ('\u{1D50E}', r#"\ensuremath{\mathfrak{K}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL K
    ('\u{1D50F}', r#"\ensuremath{\mathfrak{L}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL L
    ('\u{1D510}', r#"\ensuremath{\mathfrak{M}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL M
    ('\u{1D511}', r#"\ensuremath{\mathfrak{N}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL N
    ('\u{1D512}', r#"\ensuremath{\mathfrak{O}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL O
    ('\u{1D513}', r#"\ensuremath{\mathfrak{P}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL P
    ('\u{1D514}', r#"\ensuremath{\mathfrak{Q}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL Q
    ('\u{1D515}', r#"\ensuremath{\mathfrak{R}}"#, AMSSYMB), // (unnamed code point)
    ('\u{1D516}', r#"\ensuremath{\mathfrak{S}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL S
    ('\u{1D517}', r#"\ensuremath{\mathfrak{T}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL T
    ('\u{1D518}', r#"\ensuremath{\mathfrak{U}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL U
    ('\u{1D519}', r#"\ensuremath{\mathfrak{V}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL V
    ('\u{1D51A}', r#"\ensuremath{\mathfrak{W}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL W
    ('\u{1D51B}', r#"\ensuremath{\mathfrak{X}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL X
    ('\u{1D51C}', r#"\ensuremath{\mathfrak{Y}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR CAPITAL Y
    ('\u{1D51D}', r#"\ensuremath{\mathfrak{Z}}"#, AMSSYMB), // (unnamed code point)
    ('\u{1D51E}', r#"\ensuremath{\mathfrak{a}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL A
    ('\u{1D51F}', r#"\ensuremath{\mathfrak{b}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL B
    ('\u{1D520}', r#"\ensuremath{\mathfrak{c}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL C
    ('\u{1D521}', r#"\ensuremath{\mathfrak{d}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL D
    ('\u{1D522}', r#"\ensuremath{\mathfrak{e}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL E
    ('\u{1D523}', r#"\ensuremath{\mathfrak{f}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL F
    ('\u{1D524}', r#"\ensuremath{\mathfrak{g}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL G
    ('\u{1D525}', r#"\ensuremath{\mathfrak{h}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL H
    ('\u{1D526}', r#"\ensuremath{\mathfrak{i}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL I
    ('\u{1D527}', r#"\ensuremath{\mathfrak{j}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL J
    ('\u{1D528}', r#"\ensuremath{\mathfrak{k}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL K
    ('\u{1D529}', r#"\ensuremath{\mathfrak{l}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL L
    ('\u{1D52A}', r#"\ensuremath{\mathfrak{m}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL M
    ('\u{1D52B}', r#"\ensuremath{\mathfrak{n}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL N
    ('\u{1D52C}', r#"\ensuremath{\mathfrak{o}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL O
    ('\u{1D52D}', r#"\ensuremath{\mathfrak{p}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL P
    ('\u{1D52E}', r#"\ensuremath{\mathfrak{q}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL Q
    ('\u{1D52F}', r#"\ensuremath{\mathfrak{r}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL R
    ('\u{1D530}', r#"\ensuremath{\mathfrak{s}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL S
    ('\u{1D531}', r#"\ensuremath{\mathfrak{t}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL T
    ('\u{1D532}', r#"\ensuremath{\mathfrak{u}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL U
    ('\u{1D533}', r#"\ensuremath{\mathfrak{v}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL V
    ('\u{1D534}', r#"\ensuremath{\mathfrak{w}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL W
    ('\u{1D535}', r#"\ensuremath{\mathfrak{x}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL X
    ('\u{1D536}', r#"\ensuremath{\mathfrak{y}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL Y
    ('\u{1D537}', r#"\ensuremath{\mathfrak{z}}"#, AMSSYMB), // MATHEMATICAL FRAKTUR SMALL Z
    ('\u{1D538}', r#"\ensuremath{\mathbb{A}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL A
    ('\u{1D539}', r#"\ensuremath{\mathbb{B}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL B
    ('\u{1D53A}', r#"\ensuremath{\mathbb{C}}"#, AMSSYMB), // (unnamed code point)
    ('\u{1D53B}', r#"\ensuremath{\mathbb{D}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL D
    ('\u{1D53C}', r#"\ensuremath{\mathbb{E}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL E
    ('\u{1D53D}', r#"\ensuremath{\mathbb{F}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL F
    ('\u{1D53E}', r#"\ensuremath{\mathbb{G}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL G
    ('\u{1D53F}', r#"\ensuremath{\mathbb{H}}"#, AMSSYMB), // (unnamed code point)
    ('\u{1D540}', r#"\ensuremath{\mathbb{I}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL I
    ('\u{1D541}', r#"\ensuremath{\mathbb{J}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL J
    ('\u{1D542}', r#"\ensuremath{\mathbb{K}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL K
    ('\u{1D543}', r#"\ensuremath{\mathbb{L}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL L
    ('\u{1D544}', r#"\ensuremath{\mathbb{M}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL M
    ('\u{1D545}', r#"\ensuremath{\mathbb{N}}"#, AMSSYMB), // (unnamed code point)
    ('\u{1D546}', r#"\ensuremath{\mathbb{O}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL O
    ('\u{1D547}', r#"\ensuremath{\mathbb{P}}"#, AMSSYMB), // (unnamed code point)
    ('\u{1D548}', r#"\ensuremath{\mathbb{Q}}"#, AMSSYMB), // (unnamed code point)
    ('\u{1D549}', r#"\ensuremath{\mathbb{R}}"#, AMSSYMB), // (unnamed code point)
    ('\u{1D54A}', r#"\ensuremath{\mathbb{S}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL S
    ('\u{1D54B}', r#"\ensuremath{\mathbb{T}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL T
    ('\u{1D54C}', r#"\ensuremath{\mathbb{U}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL U
    ('\u{1D54D}', r#"\ensuremath{\mathbb{V}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL V
    ('\u{1D54E}', r#"\ensuremath{\mathbb{W}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL W
    ('\u{1D54F}', r#"\ensuremath{\mathbb{X}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL X
    ('\u{1D550}', r#"\ensuremath{\mathbb{Y}}"#, AMSSYMB), // MATHEMATICAL DOUBLE-STRUCK CAPITAL Y
    ('\u{1D551}', r#"\ensuremath{\mathbb{Z}}"#, AMSSYMB), // (unnamed code point)
    ('\u{1D552}', r#"\ensuremath{\mathbbm{a}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL A
    ('\u{1D553}', r#"\ensuremath{\mathbbm{b}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL B
    ('\u{1D554}', r#"\ensuremath{\mathbbm{c}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL C
    ('\u{1D555}', r#"\ensuremath{\mathbbm{d}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL D
    ('\u{1D556}', r#"\ensuremath{\mathbbm{e}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL E
    ('\u{1D557}', r#"\ensuremath{\mathbbm{f}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL F
    ('\u{1D558}', r#"\ensuremath{\mathbbm{g}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL G
    ('\u{1D559}', r#"\ensuremath{\mathbbm{h}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL H
    ('\u{1D55A}', r#"\ensuremath{\mathbbm{i}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL I
    ('\u{1D55B}', r#"\ensuremath{\mathbbm{j}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL J
    ('\u{1D55C}', r#"\ensuremath{\mathbbm{k}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL K
    ('\u{1D55D}', r#"\ensuremath{\mathbbm{l}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL L
    ('\u{1D55E}', r#"\ensuremath{\mathbbm{m}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL M
    ('\u{1D55F}', r#"\ensuremath{\mathbbm{n}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL N
    ('\u{1D560}', r#"\ensuremath{\mathbbm{o}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL O
    ('\u{1D561}', r#"\ensuremath{\mathbbm{p}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL P
    ('\u{1D562}', r#"\ensuremath{\mathbbm{q}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL Q
    ('\u{1D563}', r#"\ensuremath{\mathbbm{r}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL R
    ('\u{1D564}', r#"\ensuremath{\mathbbm{s}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL S
    ('\u{1D565}', r#"\ensuremath{\mathbbm{t}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL T
    ('\u{1D566}', r#"\ensuremath{\mathbbm{u}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL U
    ('\u{1D567}', r#"\ensuremath{\mathbbm{v}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL V
    ('\u{1D568}', r#"\ensuremath{\mathbbm{w}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL W
    ('\u{1D569}', r#"\ensuremath{\mathbbm{x}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL X
    ('\u{1D56A}', r#"\ensuremath{\mathbbm{y}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL Y
    ('\u{1D56B}', r#"\ensuremath{\mathbbm{z}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK SMALL Z
    ('\u{1D5A0}', r#"\ensuremath{\mathsf{A}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL A
    ('\u{1D5A1}', r#"\ensuremath{\mathsf{B}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL B
    ('\u{1D5A2}', r#"\ensuremath{\mathsf{C}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL C
    ('\u{1D5A3}', r#"\ensuremath{\mathsf{D}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL D
    ('\u{1D5A4}', r#"\ensuremath{\mathsf{E}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL E
    ('\u{1D5A5}', r#"\ensuremath{\mathsf{F}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL F
    ('\u{1D5A6}', r#"\ensuremath{\mathsf{G}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL G
    ('\u{1D5A7}', r#"\ensuremath{\mathsf{H}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL H
    ('\u{1D5A8}', r#"\ensuremath{\mathsf{I}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL I
    ('\u{1D5A9}', r#"\ensuremath{\mathsf{J}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL J
    ('\u{1D5AA}', r#"\ensuremath{\mathsf{K}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL K
    ('\u{1D5AB}', r#"\ensuremath{\mathsf{L}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL L
    ('\u{1D5AC}', r#"\ensuremath{\mathsf{M}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL M
    ('\u{1D5AD}', r#"\ensuremath{\mathsf{N}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL N
    ('\u{1D5AE}', r#"\ensuremath{\mathsf{O}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL O
    ('\u{1D5AF}', r#"\ensuremath{\mathsf{P}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL P
    ('\u{1D5B0}', r#"\ensuremath{\mathsf{Q}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL Q
    ('\u{1D5B1}', r#"\ensuremath{\mathsf{R}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL R
    ('\u{1D5B2}', r#"\ensuremath{\mathsf{S}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL S
    ('\u{1D5B3}', r#"\ensuremath{\mathsf{T}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL T
    ('\u{1D5B4}', r#"\ensuremath{\mathsf{U}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL U
    ('\u{1D5B5}', r#"\ensuremath{\mathsf{V}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL V
    ('\u{1D5B6}', r#"\ensuremath{\mathsf{W}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL W
    ('\u{1D5B7}', r#"\ensuremath{\mathsf{X}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL X
    ('\u{1D5B8}', r#"\ensuremath{\mathsf{Y}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL Y
    ('\u{1D5B9}', r#"\ensuremath{\mathsf{Z}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF CAPITAL Z
    ('\u{1D5BA}', r#"\ensuremath{\mathsf{a}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL A
    ('\u{1D5BB}', r#"\ensuremath{\mathsf{b}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL B
    ('\u{1D5BC}', r#"\ensuremath{\mathsf{c}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL C
    ('\u{1D5BD}', r#"\ensuremath{\mathsf{d}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL D
    ('\u{1D5BE}', r#"\ensuremath{\mathsf{e}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL E
    ('\u{1D5BF}', r#"\ensuremath{\mathsf{f}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL F
    ('\u{1D5C0}', r#"\ensuremath{\mathsf{g}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL G
    ('\u{1D5C1}', r#"\ensuremath{\mathsf{h}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL H
    ('\u{1D5C2}', r#"\ensuremath{\mathsf{i}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL I
    ('\u{1D5C3}', r#"\ensuremath{\mathsf{j}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL J
    ('\u{1D5C4}', r#"\ensuremath{\mathsf{k}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL K
    ('\u{1D5C5}', r#"\ensuremath{\mathsf{l}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL L
    ('\u{1D5C6}', r#"\ensuremath{\mathsf{m}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL M
    ('\u{1D5C7}', r#"\ensuremath{\mathsf{n}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL N
    ('\u{1D5C8}', r#"\ensuremath{\mathsf{o}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL O
    ('\u{1D5C9}', r#"\ensuremath{\mathsf{p}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL P
    ('\u{1D5CA}', r#"\ensuremath{\mathsf{q}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL Q
    ('\u{1D5CB}', r#"\ensuremath{\mathsf{r}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL R
    ('\u{1D5CC}', r#"\ensuremath{\mathsf{s}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL S
    ('\u{1D5CD}', r#"\ensuremath{\mathsf{t}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL T
    ('\u{1D5CE}', r#"\ensuremath{\mathsf{u}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL U
    ('\u{1D5CF}', r#"\ensuremath{\mathsf{v}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL V
    ('\u{1D5D0}', r#"\ensuremath{\mathsf{w}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL W
    ('\u{1D5D1}', r#"\ensuremath{\mathsf{x}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL X
    ('\u{1D5D2}', r#"\ensuremath{\mathsf{y}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL Y
    ('\u{1D5D3}', r#"\ensuremath{\mathsf{z}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF SMALL Z
    ('\u{1D670}', r#"\ensuremath{\mathtt{A}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL A
    ('\u{1D671}', r#"\ensuremath{\mathtt{B}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL B
    ('\u{1D672}', r#"\ensuremath{\mathtt{C}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL C
    ('\u{1D673}', r#"\ensuremath{\mathtt{D}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL D
    ('\u{1D674}', r#"\ensuremath{\mathtt{E}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL E
    ('\u{1D675}', r#"\ensuremath{\mathtt{F}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL F
    ('\u{1D676}', r#"\ensuremath{\mathtt{G}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL G
    ('\u{1D677}', r#"\ensuremath{\mathtt{H}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL H
    ('\u{1D678}', r#"\ensuremath{\mathtt{I}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL I
    ('\u{1D679}', r#"\ensuremath{\mathtt{J}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL J
    ('\u{1D67A}', r#"\ensuremath{\mathtt{K}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL K
    ('\u{1D67B}', r#"\ensuremath{\mathtt{L}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL L
    ('\u{1D67C}', r#"\ensuremath{\mathtt{M}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL M
    ('\u{1D67D}', r#"\ensuremath{\mathtt{N}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL N
    ('\u{1D67E}', r#"\ensuremath{\mathtt{O}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL O
    ('\u{1D67F}', r#"\ensuremath{\mathtt{P}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL P
    ('\u{1D680}', r#"\ensuremath{\mathtt{Q}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL Q
    ('\u{1D681}', r#"\ensuremath{\mathtt{R}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL R
    ('\u{1D682}', r#"\ensuremath{\mathtt{S}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL S
    ('\u{1D683}', r#"\ensuremath{\mathtt{T}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL T
    ('\u{1D684}', r#"\ensuremath{\mathtt{U}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL U
    ('\u{1D685}', r#"\ensuremath{\mathtt{V}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL V
    ('\u{1D686}', r#"\ensuremath{\mathtt{W}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL W
    ('\u{1D687}', r#"\ensuremath{\mathtt{X}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL X
    ('\u{1D688}', r#"\ensuremath{\mathtt{Y}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL Y
    ('\u{1D689}', r#"\ensuremath{\mathtt{Z}}"#, BUILTINS), // MATHEMATICAL MONOSPACE CAPITAL Z
    ('\u{1D68A}', r#"\ensuremath{\mathtt{a}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL A
    ('\u{1D68B}', r#"\ensuremath{\mathtt{b}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL B
    ('\u{1D68C}', r#"\ensuremath{\mathtt{c}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL C
    ('\u{1D68D}', r#"\ensuremath{\mathtt{d}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL D
    ('\u{1D68E}', r#"\ensuremath{\mathtt{e}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL E
    ('\u{1D68F}', r#"\ensuremath{\mathtt{f}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL F
    ('\u{1D690}', r#"\ensuremath{\mathtt{g}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL G
    ('\u{1D691}', r#"\ensuremath{\mathtt{h}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL H
    ('\u{1D692}', r#"\ensuremath{\mathtt{i}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL I
    ('\u{1D693}', r#"\ensuremath{\mathtt{j}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL J
    ('\u{1D694}', r#"\ensuremath{\mathtt{k}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL K
    ('\u{1D695}', r#"\ensuremath{\mathtt{l}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL L
    ('\u{1D696}', r#"\ensuremath{\mathtt{m}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL M
    ('\u{1D697}', r#"\ensuremath{\mathtt{n}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL N
    ('\u{1D698}', r#"\ensuremath{\mathtt{o}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL O
    ('\u{1D699}', r#"\ensuremath{\mathtt{p}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL P
    ('\u{1D69A}', r#"\ensuremath{\mathtt{q}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL Q
    ('\u{1D69B}', r#"\ensuremath{\mathtt{r}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL R
    ('\u{1D69C}', r#"\ensuremath{\mathtt{s}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL S
    ('\u{1D69D}', r#"\ensuremath{\mathtt{t}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL T
    ('\u{1D69E}', r#"\ensuremath{\mathtt{u}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL U
    ('\u{1D69F}', r#"\ensuremath{\mathtt{v}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL V
    ('\u{1D6A0}', r#"\ensuremath{\mathtt{w}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL W
    ('\u{1D6A1}', r#"\ensuremath{\mathtt{x}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL X
    ('\u{1D6A2}', r#"\ensuremath{\mathtt{y}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL Y
    ('\u{1D6A3}', r#"\ensuremath{\mathtt{z}}"#, BUILTINS), // MATHEMATICAL MONOSPACE SMALL Z
    ('\u{1D7CE}', r#"\ensuremath{\mathbf{0}}"#, BUILTINS), // MATHEMATICAL BOLD DIGIT ZERO
    ('\u{1D7CF}', r#"\ensuremath{\mathbf{1}}"#, BUILTINS), // MATHEMATICAL BOLD DIGIT ONE
    ('\u{1D7D0}', r#"\ensuremath{\mathbf{2}}"#, BUILTINS), // MATHEMATICAL BOLD DIGIT TWO
    ('\u{1D7D1}', r#"\ensuremath{\mathbf{3}}"#, BUILTINS), // MATHEMATICAL BOLD DIGIT THREE
    ('\u{1D7D2}', r#"\ensuremath{\mathbf{4}}"#, BUILTINS), // MATHEMATICAL BOLD DIGIT FOUR
    ('\u{1D7D3}', r#"\ensuremath{\mathbf{5}}"#, BUILTINS), // MATHEMATICAL BOLD DIGIT FIVE
    ('\u{1D7D4}', r#"\ensuremath{\mathbf{6}}"#, BUILTINS), // MATHEMATICAL BOLD DIGIT SIX
    ('\u{1D7D5}', r#"\ensuremath{\mathbf{7}}"#, BUILTINS), // MATHEMATICAL BOLD DIGIT SEVEN
    ('\u{1D7D6}', r#"\ensuremath{\mathbf{8}}"#, BUILTINS), // MATHEMATICAL BOLD DIGIT EIGHT
    ('\u{1D7D7}', r#"\ensuremath{\mathbf{9}}"#, BUILTINS), // MATHEMATICAL BOLD DIGIT NINE
    ('\u{1D7D8}', r#"\ensuremath{\flmBbold{0}}"#, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT ZERO
    ('\u{1D7D9}', r#"\ensuremath{\mathds{1}}"#, DSFONT), // MATHEMATICAL DOUBLE-STRUCK DIGIT ONE
    ('\u{1D7DA}', r#"\ensuremath{\mathbbm{2}}"#, BBM), // MATHEMATICAL DOUBLE-STRUCK DIGIT TWO
    ('\u{1D7DB}', r#"\ensuremath{\flmBbold{3}}"#, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT THREE
    ('\u{1D7DC}', r#"\ensuremath{\flmBbold{4}}"#, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT FOUR
    ('\u{1D7DD}', r#"\ensuremath{\flmBbold{5}}"#, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT FIVE
    ('\u{1D7DE}', r#"\ensuremath{\flmBbold{6}}"#, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT SIX
    ('\u{1D7DF}', r#"\ensuremath{\flmBbold{7}}"#, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT SEVEN
    ('\u{1D7E0}', r#"\ensuremath{\flmBbold{8}}"#, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT EIGHT
    ('\u{1D7E1}', r#"\ensuremath{\flmBbold{9}}"#, BBOLD), // MATHEMATICAL DOUBLE-STRUCK DIGIT NINE
    ('\u{1D7E2}', r#"\ensuremath{\mathsf{0}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT ZERO
    ('\u{1D7E3}', r#"\ensuremath{\mathsf{1}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT ONE
    ('\u{1D7E4}', r#"\ensuremath{\mathsf{2}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT TWO
    ('\u{1D7E5}', r#"\ensuremath{\mathsf{3}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT THREE
    ('\u{1D7E6}', r#"\ensuremath{\mathsf{4}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT FOUR
    ('\u{1D7E7}', r#"\ensuremath{\mathsf{5}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT FIVE
    ('\u{1D7E8}', r#"\ensuremath{\mathsf{6}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT SIX
    ('\u{1D7E9}', r#"\ensuremath{\mathsf{7}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT SEVEN
    ('\u{1D7EA}', r#"\ensuremath{\mathsf{8}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT EIGHT
    ('\u{1D7EB}', r#"\ensuremath{\mathsf{9}}"#, BUILTINS), // MATHEMATICAL SANS-SERIF DIGIT NINE
    ('\u{1D7F6}', r#"\ensuremath{\mathtt{0}}"#, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT ZERO
    ('\u{1D7F7}', r#"\ensuremath{\mathtt{1}}"#, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT ONE
    ('\u{1D7F8}', r#"\ensuremath{\mathtt{2}}"#, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT TWO
    ('\u{1D7F9}', r#"\ensuremath{\mathtt{3}}"#, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT THREE
    ('\u{1D7FA}', r#"\ensuremath{\mathtt{4}}"#, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT FOUR
    ('\u{1D7FB}', r#"\ensuremath{\mathtt{5}}"#, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT FIVE
    ('\u{1D7FC}', r#"\ensuremath{\mathtt{6}}"#, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT SIX
    ('\u{1D7FD}', r#"\ensuremath{\mathtt{7}}"#, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT SEVEN
    ('\u{1D7FE}', r#"\ensuremath{\mathtt{8}}"#, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT EIGHT
    ('\u{1D7FF}', r#"\ensuremath{\mathtt{9}}"#, BUILTINS), // MATHEMATICAL MONOSPACE DIGIT NINE
];

pub const static TABLE : StaticTable = compile_static_table!(ENTRIES, two_level_linear);
