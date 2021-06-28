#[cfg(test)]
mod tests;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[allow(clippy::struct_excessive_bools)]
pub struct PinyinParser {
    _strict: bool,
    _preserve_punctuations: bool,
    _preserve_spaces: bool,
    _preserve_capitalization: bool,
}

impl Default for PinyinParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PinyinParser {
    pub fn new() -> Self {
        PinyinParser {
            _strict: false,
            _preserve_spaces: false,
            _preserve_capitalization: false,
            _preserve_punctuations: false,
        }
    }

    pub fn is_strict(self, b: bool) -> Self {
        Self { _strict: b, ..self }
    }

    pub fn preserve_spaces(self, b: bool) -> Self {
        Self {
            _preserve_spaces: b,
            ..self
        }
    }

    pub fn preserve_capitalization(self, b: bool) -> Self {
        Self {
            _preserve_capitalization: b,
            ..self
        }
    }

    /// allow british spelling
    pub fn preserve_capitalisation(self, b: bool) -> Self {
        self.preserve_capitalization(b)
    }

    pub fn preserve_punctuations(self, b: bool) -> Self {
        Self {
            _preserve_spaces: b,
            ..self
        }
    }

    pub fn parse(self, s: &str) -> PinyinParserIter {
        PinyinParserIter {
            configs: self,
            it: VecAndIndex {
                vec: UnicodeSegmentation::graphemes(s, true)
                    .map(|c| pinyin_token::to_token(c))
                    .collect::<Vec<_>>(),
                next_pos: 0,
            },
            state: ParserState::BeforeWordInitial,
        }
    }

    pub fn strict(s: &str) -> PinyinParserIter {
        Self::new().is_strict(true).parse(s)
    }

    pub fn loose(s: &str) -> PinyinParserIter {
        Self::new().parse(s)
    }
}

mod pinyin_token;

struct VecAndIndex<T> {
    vec: std::vec::Vec<T>,
    next_pos: usize,
}

pub struct PinyinParserIter {
    configs: PinyinParser,
    it: VecAndIndex<pinyin_token::PinyinToken>,
    state: ParserState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ParserState {
    BeforeWordInitial,
    InitialParsed(SpellingInitial),
    ZCSParsed(ZCS),
}

impl<T> VecAndIndex<T> {
    fn read_next_token(&mut self) -> Option<&T> {
        self.vec.get(self.next_pos)
    }

    fn backtrack(&mut self) {
        self.next_pos -= 1;
    }
}

impl Iterator for PinyinParserIter {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        use pinyin_token::Alphabet;
        use pinyin_token::PinyinToken::*;
        use ParserState::*;
        loop {
            match (self.it.read_next_token(), self.state) {
                (None, BeforeWordInitial) => return None,
                (Some(Punctuation(s)), BeforeWordInitial) => {
                    if self.configs._preserve_punctuations {
                        return Some((*s).to_owned());
                    } else {
                        continue;
                    }
                }
                (Some(Space(s)), BeforeWordInitial) => {
                    if self.configs._preserve_spaces {
                        return Some((*s).to_owned());
                    } else {
                        continue;
                    }
                }

                (Some(Alph(alph)), BeforeWordInitial) => match alph.alphabet {
                    Alphabet::B => self.state = InitialParsed(SpellingInitial::B),
                    Alphabet::P => self.state = InitialParsed(SpellingInitial::P),
                    Alphabet::M => {
                        if alph.diacritics.is_empty() {
                            self.state = InitialParsed(SpellingInitial::M);
                        } else {
                            return Some(alph.to_str(
                                self.configs._preserve_capitalization,
                                self.configs._strict,
                            ));
                        }
                    }
                    Alphabet::F => self.state = InitialParsed(SpellingInitial::F),
                    Alphabet::D => self.state = InitialParsed(SpellingInitial::D),
                    Alphabet::T => self.state = InitialParsed(SpellingInitial::T),
                    Alphabet::N => {
                        if alph.diacritics.is_empty() {
                            self.state = InitialParsed(SpellingInitial::N)
                        } else {
                            return Some(alph.to_str(
                                self.configs._preserve_capitalization,
                                self.configs._strict,
                            ));
                        }
                    }
                    Alphabet::L => self.state = InitialParsed(SpellingInitial::L),
                    Alphabet::G => self.state = InitialParsed(SpellingInitial::G),
                    Alphabet::K => self.state = InitialParsed(SpellingInitial::K),
                    Alphabet::H => self.state = InitialParsed(SpellingInitial::H),
                    Alphabet::J => self.state = InitialParsed(SpellingInitial::J),
                    Alphabet::Q => self.state = InitialParsed(SpellingInitial::Q),
                    Alphabet::X => self.state = InitialParsed(SpellingInitial::X),
                    Alphabet::R => self.state = InitialParsed(SpellingInitial::R),
                    Alphabet::Y => self.state = InitialParsed(SpellingInitial::Y),
                    Alphabet::W => self.state = InitialParsed(SpellingInitial::W),
                    Alphabet::Z => {
                        if alph.diacritics.is_empty() {
                            self.state = ZCSParsed(ZCS::Z)
                        } else if matches!(
                            &alph.diacritics[..],
                            &[pinyin_token::Diacritic::Circumflex]
                        ) {
                            self.state = InitialParsed(SpellingInitial::ZH)
                        } else {
                            return Some(alph.to_str(
                                self.configs._preserve_capitalization,
                                self.configs._strict,
                            ));
                        }
                    }
                    Alphabet::C => {
                        if alph.diacritics.is_empty() {
                            self.state = ZCSParsed(ZCS::C)
                        } else if matches!(
                            &alph.diacritics[..],
                            &[pinyin_token::Diacritic::Circumflex]
                        ) {
                            self.state = InitialParsed(SpellingInitial::CH)
                        } else {
                            return Some(alph.to_str(
                                self.configs._preserve_capitalization,
                                self.configs._strict,
                            ));
                        }
                    }
                    Alphabet::S => {
                        if alph.diacritics.is_empty() {
                            self.state = ZCSParsed(ZCS::S)
                        } else if matches!(
                            &alph.diacritics[..],
                            &[pinyin_token::Diacritic::Circumflex]
                        ) {
                            self.state = InitialParsed(SpellingInitial::SH)
                        } else {
                            return Some(alph.to_str(
                                self.configs._preserve_capitalization,
                                self.configs._strict,
                            ));
                        }
                    }
                    Alphabet::A | Alphabet::E | Alphabet::O => {
                        self.it.backtrack();
                        self.state = InitialParsed(SpellingInitial::ZeroAEO);
                    }

                    Alphabet::I => todo!(),
                    Alphabet::U => todo!(),
                    Alphabet::Ŋ => todo!(),
                },

                (Some(Alph(alph)), ZCSParsed(zcs)) => match alph.alphabet {
                    Alphabet::H => {
                        self.state = match zcs {
                            ZCS::Z => InitialParsed(SpellingInitial::ZH),
                            ZCS::C => InitialParsed(SpellingInitial::CH),
                            ZCS::S => InitialParsed(SpellingInitial::SH),
                        }
                    }
                    _ => {
                        self.it.backtrack();
                        self.state = match zcs {
                            ZCS::Z => InitialParsed(SpellingInitial::Z),
                            ZCS::C => InitialParsed(SpellingInitial::C),
                            ZCS::S => InitialParsed(SpellingInitial::S),
                        }
                    }
                },

                _ => todo!(),
            }
        }
    }
}

mod finals;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ZCS {
    Z,
    C,
    S,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SpellingInitial {
    B,
    P,
    M,
    F,
    D,
    T,
    N,
    L,
    G,
    K,
    H,
    J,
    Q,
    X,
    ZH,
    CH,
    SH,
    R,
    Z,
    C,
    S,
    Y,
    W,
    ZeroAEO,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct PinyinAmbiguousParser {
    _preserve_punctuations: bool,
    _preserve_spaces: bool,
    _preserve_capitalization: bool,
}

impl Default for PinyinAmbiguousParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PinyinAmbiguousParser {
    pub fn new() -> Self {
        PinyinAmbiguousParser {
            _preserve_spaces: false,
            _preserve_capitalization: false,
            _preserve_punctuations: false,
        }
    }

    pub fn preserve_spaces(self, b: bool) -> Self {
        Self {
            _preserve_spaces: b,
            ..self
        }
    }

    pub fn preserve_capitalization(self, b: bool) -> Self {
        Self {
            _preserve_capitalization: b,
            ..self
        }
    }

    /// allow british spelling
    pub fn preserve_capitalisation(self, b: bool) -> Self {
        self.preserve_capitalization(b)
    }

    pub fn preserve_punctuations(self, b: bool) -> Self {
        Self {
            _preserve_spaces: b,
            ..self
        }
    }

    pub fn parse(self, s: &str) -> PinyinAmbiguousParserIter {
        PinyinAmbiguousParserIter {
            configs: self,
            it: VecAndIndex {
                vec: UnicodeSegmentation::graphemes(s, true)
                    .map(|c| pinyin_token::to_token(c))
                    .collect::<Vec<_>>(),
                next_pos: 0,
            },
            state: ParserState::BeforeWordInitial,
        }
    }
}

pub struct PinyinAmbiguousParserIter {
    configs: PinyinAmbiguousParser,
    it: VecAndIndex<pinyin_token::PinyinToken>,
    state: ParserState,
}
