use strum::EnumIter;
use winit::keyboard::{self, PhysicalKey};

// Stolen from winit as a base to start from
#[derive(EnumIter, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyCode {
  Unknown,
  /// <kbd>`</kbd> on a US keyboard. This is also called a backtick or grave.
  /// This is the <kbd>半角</kbd>/<kbd>全角</kbd>/<kbd>漢字</kbd>
  /// (hankaku/zenkaku/kanji) key on Japanese keyboards
  Backquote,
  /// Used for both the US <kbd>\\</kbd> (on the 101-key layout) and also for
  /// the key located between the <kbd>"</kbd> and <kbd>Enter</kbd> keys on
  /// row C of the 102-, 104- and 106-key layouts.
  /// Labeled <kbd>#</kbd> on a UK (102) keyboard.
  Backslash,
  /// <kbd>[</kbd> on a US keyboard.
  BracketLeft,
  /// <kbd>]</kbd> on a US keyboard.
  BracketRight,
  /// <kbd>,</kbd> on a US keyboard.
  Comma,
  /// <kbd>0</kbd> on a US keyboard.
  _0,
  /// <kbd>1</kbd> on a US keyboard.
  _1,
  /// <kbd>2</kbd> on a US keyboard.
  _2,
  /// <kbd>3</kbd> on a US keyboard.
  _3,
  /// <kbd>4</kbd> on a US keyboard.
  _4,
  /// <kbd>5</kbd> on a US keyboard.
  _5,
  /// <kbd>6</kbd> on a US keyboard.
  _6,
  /// <kbd>7</kbd> on a US keyboard.
  _7,
  /// <kbd>8</kbd> on a US keyboard.
  _8,
  /// <kbd>9</kbd> on a US keyboard.
  _9,
  /// <kbd>=</kbd> on a US keyboard.
  Equal,
  /// Located between the left <kbd>Shift</kbd> and <kbd>Z</kbd> keys.
  /// Labeled <kbd>\\</kbd> on a UK keyboard.
  IntlBackslash,
  /// Located between the <kbd>/</kbd> and right <kbd>Shift</kbd> keys.
  /// Labeled <kbd>\\</kbd> (ro) on a Japanese keyboard.
  IntlRo,
  /// Located between the <kbd>=</kbd> and <kbd>Backspace</kbd> keys.
  /// Labeled <kbd>¥</kbd> (yen) on a Japanese keyboard. <kbd>\\</kbd> on a
  /// Russian keyboard.
  IntlYen,
  /// <kbd>a</kbd> on a US keyboard.
  /// Labeled <kbd>q</kbd> on an AZERTY (e.g., French) keyboard.
  A,
  /// <kbd>b</kbd> on a US keyboard.
  B,
  /// <kbd>c</kbd> on a US keyboard.
  C,
  /// <kbd>d</kbd> on a US keyboard.
  D,
  /// <kbd>e</kbd> on a US keyboard.
  E,
  /// <kbd>f</kbd> on a US keyboard.
  F,
  /// <kbd>g</kbd> on a US keyboard.
  G,
  /// <kbd>h</kbd> on a US keyboard.
  H,
  /// <kbd>i</kbd> on a US keyboard.
  I,
  /// <kbd>j</kbd> on a US keyboard.
  J,
  /// <kbd>k</kbd> on a US keyboard.
  K,
  /// <kbd>l</kbd> on a US keyboard.
  L,
  /// <kbd>m</kbd> on a US keyboard.
  M,
  /// <kbd>n</kbd> on a US keyboard.
  N,
  /// <kbd>o</kbd> on a US keyboard.
  O,
  /// <kbd>p</kbd> on a US keyboard.
  P,
  /// <kbd>q</kbd> on a US keyboard.
  /// Labeled <kbd>a</kbd> on an AZERTY (e.g., French) keyboard.
  Q,
  /// <kbd>r</kbd> on a US keyboard.
  R,
  /// <kbd>s</kbd> on a US keyboard.
  S,
  /// <kbd>t</kbd> on a US keyboard.
  T,
  /// <kbd>u</kbd> on a US keyboard.
  U,
  /// <kbd>v</kbd> on a US keyboard.
  V,
  /// <kbd>w</kbd> on a US keyboard.
  /// Labeled <kbd>z</kbd> on an AZERTY (e.g., French) keyboard.
  W,
  /// <kbd>x</kbd> on a US keyboard.
  X,
  /// <kbd>y</kbd> on a US keyboard.
  /// Labeled <kbd>z</kbd> on a QWERTZ (e.g., German) keyboard.
  Y,
  /// <kbd>z</kbd> on a US keyboard.
  /// Labeled <kbd>w</kbd> on an AZERTY (e.g., French) keyboard, and
  /// <kbd>y</kbd> on a QWERTZ (e.g., German) keyboard.
  Z,
  /// <kbd>-</kbd> on a US keyboard.
  Minus,
  /// <kbd>.</kbd> on a US keyboard.
  Period,
  /// <kbd>'</kbd> on a US keyboard.
  Quote,
  /// <kbd>;</kbd> on a US keyboard.
  Semicolon,
  /// <kbd>/</kbd> on a US keyboard.
  Slash,
  /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
  AltLeft,
  /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
  /// This is labeled <kbd>AltGr</kbd> on many keyboard layouts.
  AltRight,
  /// <kbd>Backspace</kbd> or <kbd>⌫</kbd>.
  /// Labeled <kbd>Delete</kbd> on Apple keyboards.
  Backspace,
  /// <kbd>CapsLock</kbd> or <kbd>⇪</kbd>
  CapsLock,
  /// The application context menu key, which is typically found between the
  /// right <kbd>Super</kbd> key and the right <kbd>Control</kbd> key.
  ContextMenu,
  /// <kbd>Control</kbd> or <kbd>⌃</kbd>
  ControlLeft,
  /// <kbd>Control</kbd> or <kbd>⌃</kbd>
  ControlRight,
  /// <kbd>Enter</kbd> or <kbd>↵</kbd>. Labeled <kbd>Return</kbd> on Apple
  /// keyboards.
  Enter,
  /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
  SuperLeft,
  /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
  SuperRight,
  /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
  ShiftLeft,
  /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
  ShiftRight,
  /// <kbd> </kbd> (space)
  Space,
  /// <kbd>Tab</kbd> or <kbd>⇥</kbd>
  Tab,
  /// Japanese: <kbd>変</kbd> (henkan)
  Convert,
  /// Japanese: <kbd>カタカナ</kbd>/<kbd>ひらがな</kbd>/<kbd>ローマ字</kbd>
  /// (katakana/hiragana/romaji)
  KanaMode,
  /// Korean: HangulMode <kbd>한/영</kbd> (han/yeong)
  ///
  /// Japanese (Mac keyboard): <kbd>か</kbd> (kana)
  Lang1,
  /// Korean: Hanja <kbd>한</kbd> (hanja)
  ///
  /// Japanese (Mac keyboard): <kbd>英</kbd> (eisu)
  Lang2,
  /// Japanese (word-processing keyboard): Katakana
  Lang3,
  /// Japanese (word-processing keyboard): Hiragana
  Lang4,
  /// Japanese (word-processing keyboard): Zenkaku/Hankaku
  Lang5,
  /// Japanese: <kbd>無変換</kbd> (muhenkan)
  NonConvert,
  /// <kbd>⌦</kbd>. The forward delete key.
  /// Note that on Apple keyboards, the key labelled <kbd>Delete</kbd> on the
  /// main part of the keyboard is encoded as [`Backspace`].
  ///
  /// [`Backspace`]: Self::Backspace
  Delete,
  /// <kbd>Page Down</kbd>, <kbd>End</kbd>, or <kbd>↘</kbd>
  End,
  /// <kbd>Help</kbd>. Not present on standard PC keyboards.
  Help,
  /// <kbd>Home</kbd> or <kbd>↖</kbd>
  Home,
  /// <kbd>Insert</kbd> or <kbd>Ins</kbd>. Not present on Apple keyboards.
  Insert,
  /// <kbd>Page Down</kbd>, <kbd>PgDn</kbd>, or <kbd>⇟</kbd>
  PageDown,
  /// <kbd>Page Up</kbd>, <kbd>PgUp</kbd>, or <kbd>⇞</kbd>
  PageUp,
  /// <kbd>↓</kbd>
  ArrowDown,
  /// <kbd>←</kbd>
  ArrowLeft,
  /// <kbd>→</kbd>
  ArrowRight,
  /// <kbd>↑</kbd>
  ArrowUp,
  /// On the Mac, this is used for the numpad <kbd>Clear</kbd> key.
  NumLock,
  /// <kbd>0 Ins</kbd> on a keyboard. <kbd>0</kbd> on a phone or remote control
  Numpad0,
  /// <kbd>1 End</kbd> on a keyboard. <kbd>1</kbd> or <kbd>1 QZ</kbd> on a phone
  /// or remote control
  Numpad1,
  /// <kbd>2 ↓</kbd> on a keyboard. <kbd>2 ABC</kbd> on a phone or remote
  /// control
  Numpad2,
  /// <kbd>3 PgDn</kbd> on a keyboard. <kbd>3 DEF</kbd> on a phone or remote
  /// control
  Numpad3,
  /// <kbd>4 ←</kbd> on a keyboard. <kbd>4 GHI</kbd> on a phone or remote
  /// control
  Numpad4,
  /// <kbd>5</kbd> on a keyboard. <kbd>5 JKL</kbd> on a phone or remote control
  Numpad5,
  /// <kbd>6 →</kbd> on a keyboard. <kbd>6 MNO</kbd> on a phone or remote
  /// control
  Numpad6,
  /// <kbd>7 Home</kbd> on a keyboard. <kbd>7 PQRS</kbd> or <kbd>7 PRS</kbd> on
  /// a phone or remote control
  Numpad7,
  /// <kbd>8 ↑</kbd> on a keyboard. <kbd>8 TUV</kbd> on a phone or remote
  /// control
  Numpad8,
  /// <kbd>9 PgUp</kbd> on a keyboard. <kbd>9 WXYZ</kbd> or <kbd>9 WXY</kbd> on
  /// a phone or remote control
  Numpad9,
  /// <kbd>+</kbd>
  NumpadAdd,
  /// Found on the Microsoft Natural Keyboard.
  NumpadBackspace,
  /// <kbd>C</kbd> or <kbd>A</kbd> (All Clear). Also for use with numpads that
  /// have a <kbd>Clear</kbd> key that is separate from the <kbd>NumLock</kbd>
  /// key. On the Mac, the numpad <kbd>Clear</kbd> key is encoded as
  /// [`NumLock`].
  ///
  /// [`NumLock`]: Self::NumLock
  NumpadClear,
  /// <kbd>C</kbd> (Clear Entry)
  NumpadClearEntry,
  /// <kbd>,</kbd> (thousands separator). For locales where the thousands
  /// separator is a "." (e.g., Brazil), this key may generate a <kbd>.</kbd>.
  NumpadComma,
  /// <kbd>. Del</kbd>. For locales where the decimal separator is "," (e.g.,
  /// Brazil), this key may generate a <kbd>,</kbd>.
  NumpadDecimal,
  /// <kbd>/</kbd>
  NumpadDivide,
  NumpadEnter,
  /// <kbd>=</kbd>
  NumpadEqual,
  /// <kbd>#</kbd> on a phone or remote control device. This key is typically
  /// found below the <kbd>9</kbd> key and to the right of the <kbd>0</kbd>
  /// key.
  NumpadHash,
  /// <kbd>M</kbd> Add current entry to the value stored in memory.
  NumpadMemoryAdd,
  /// <kbd>M</kbd> Clear the value stored in memory.
  NumpadMemoryClear,
  /// <kbd>M</kbd> Replace the current entry with the value stored in memory.
  NumpadMemoryRecall,
  /// <kbd>M</kbd> Replace the value stored in memory with the current entry.
  NumpadMemoryStore,
  /// <kbd>M</kbd> Subtract current entry from the value stored in memory.
  NumpadMemorySubtract,
  /// <kbd>*</kbd> on a keyboard. For use with numpads that provide mathematical
  /// operations (<kbd>+</kbd>, <kbd>-</kbd> <kbd>*</kbd> and <kbd>/</kbd>).
  ///
  /// Use `NumpadStar` for the <kbd>*</kbd> key on phones and remote controls.
  NumpadMultiply,
  /// <kbd>(</kbd> Found on the Microsoft Natural Keyboard.
  NumpadParenLeft,
  /// <kbd>)</kbd> Found on the Microsoft Natural Keyboard.
  NumpadParenRight,
  /// <kbd>*</kbd> on a phone or remote control device.
  ///
  /// This key is typically found below the <kbd>7</kbd> key and to the left of
  /// the <kbd>0</kbd> key.
  ///
  /// Use <kbd>"NumpadMultiply"</kbd> for the <kbd>*</kbd> key on
  /// numeric keypads.
  NumpadStar,
  /// <kbd>-</kbd>
  NumpadSubtract,
  /// <kbd>Esc</kbd> or <kbd>⎋</kbd>
  Escape,
  /// <kbd>Fn</kbd> This is typically a hardware key that does not generate a
  /// separate code.
  Fn,
  /// <kbd>FLock</kbd> or <kbd>FnLock</kbd>. Function Lock key. Found on the
  /// Microsoft Natural Keyboard.
  FnLock,
  /// <kbd>PrtScr SysRq</kbd> or <kbd>Print Screen</kbd>
  PrintScreen,
  /// <kbd>Scroll Lock</kbd>
  ScrollLock,
  /// <kbd>Pause Break</kbd>
  Pause,
  /// Some laptops place this key to the left of the <kbd>↑</kbd> key.
  ///
  /// This also the "back" button (triangle) on Android.
  BrowserBack,
  BrowserFavorites,
  /// Some laptops place this key to the right of the <kbd>↑</kbd> key.
  BrowserForward,
  /// The "home" button on Android.
  BrowserHome,
  BrowserRefresh,
  BrowserSearch,
  BrowserStop,
  /// <kbd>Eject</kbd> or <kbd>⏏</kbd>. This key is placed in the function
  /// section on some Apple keyboards.
  Eject,
  /// Sometimes labelled <kbd>My Computer</kbd> on the keyboard
  LaunchApp1,
  /// Sometimes labelled <kbd>Calculator</kbd> on the keyboard
  LaunchApp2,
  LaunchMail,
  MediaPlayPause,
  MediaSelect,
  MediaStop,
  MediaTrackNext,
  MediaTrackPrevious,
  /// This key is placed in the function section on some Apple keyboards,
  /// replacing the <kbd>Eject</kbd> key.
  Power,
  Sleep,
  AudioVolumeDown,
  AudioVolumeMute,
  AudioVolumeUp,
  WakeUp,
  // Legacy modifier key. Also called "Super" in certain places.
  Meta,
  // Legacy modifier key.
  Hyper,
  Turbo,
  Abort,
  Resume,
  Suspend,
  /// Found on Sun’s USB keyboard.
  Again,
  /// Found on Sun’s USB keyboard.
  Copy,
  /// Found on Sun’s USB keyboard.
  Cut,
  /// Found on Sun’s USB keyboard.
  Find,
  /// Found on Sun’s USB keyboard.
  Open,
  /// Found on Sun’s USB keyboard.
  Paste,
  /// Found on Sun’s USB keyboard.
  Props,
  /// Found on Sun’s USB keyboard.
  Select,
  /// Found on Sun’s USB keyboard.
  Undo,
  /// Use for dedicated <kbd>ひらがな</kbd> key found on some Japanese word
  /// processing keyboards.
  Hiragana,
  /// Use for dedicated <kbd>カタカナ</kbd> key found on some Japanese word
  /// processing keyboards.
  Katakana,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F1,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F2,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F3,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F4,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F5,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F6,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F7,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F8,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F9,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F10,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F11,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F12,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F13,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F14,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F15,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F16,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F17,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F18,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F19,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F20,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F21,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F22,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F23,
  /// General-purpose function key.
  /// Usually found at the top of the keyboard.
  F24,
  /// General-purpose function key.
  F25,
  /// General-purpose function key.
  F26,
  /// General-purpose function key.
  F27,
  /// General-purpose function key.
  F28,
  /// General-purpose function key.
  F29,
  /// General-purpose function key.
  F30,
  /// General-purpose function key.
  F31,
  /// General-purpose function key.
  F32,
  /// General-purpose function key.
  F33,
  /// General-purpose function key.
  F34,
  /// General-purpose function key.
  F35,
}

impl From<PhysicalKey> for KeyCode {
  fn from(value: PhysicalKey) -> Self {
    let PhysicalKey::Code(value) = value else {
      return Self::Unknown;
    };

    match value {
      keyboard::KeyCode::Digit1 => KeyCode::_1,
      keyboard::KeyCode::Digit2 => KeyCode::_2,
      keyboard::KeyCode::Digit3 => KeyCode::_3,
      keyboard::KeyCode::Digit4 => KeyCode::_4,
      keyboard::KeyCode::Digit5 => KeyCode::_5,
      keyboard::KeyCode::Digit6 => KeyCode::_6,
      keyboard::KeyCode::Digit7 => KeyCode::_7,
      keyboard::KeyCode::Digit8 => KeyCode::_8,
      keyboard::KeyCode::Digit9 => KeyCode::_9,
      keyboard::KeyCode::Digit0 => KeyCode::_0,
      keyboard::KeyCode::KeyA => KeyCode::A,
      keyboard::KeyCode::KeyB => KeyCode::B,
      keyboard::KeyCode::KeyC => KeyCode::C,
      keyboard::KeyCode::KeyD => KeyCode::D,
      keyboard::KeyCode::KeyE => KeyCode::E,
      keyboard::KeyCode::KeyF => KeyCode::F,
      keyboard::KeyCode::KeyG => KeyCode::G,
      keyboard::KeyCode::KeyH => KeyCode::H,
      keyboard::KeyCode::KeyI => KeyCode::I,
      keyboard::KeyCode::KeyJ => KeyCode::J,
      keyboard::KeyCode::KeyK => KeyCode::K,
      keyboard::KeyCode::KeyL => KeyCode::L,
      keyboard::KeyCode::KeyM => KeyCode::M,
      keyboard::KeyCode::KeyN => KeyCode::N,
      keyboard::KeyCode::KeyO => KeyCode::O,
      keyboard::KeyCode::KeyP => KeyCode::P,
      keyboard::KeyCode::KeyQ => KeyCode::Q,
      keyboard::KeyCode::KeyR => KeyCode::R,
      keyboard::KeyCode::KeyS => KeyCode::S,
      keyboard::KeyCode::KeyT => KeyCode::T,
      keyboard::KeyCode::KeyU => KeyCode::U,
      keyboard::KeyCode::KeyV => KeyCode::V,
      keyboard::KeyCode::KeyW => KeyCode::W,
      keyboard::KeyCode::KeyX => KeyCode::X,
      keyboard::KeyCode::KeyY => KeyCode::Y,
      keyboard::KeyCode::KeyZ => KeyCode::Z,
      keyboard::KeyCode::Escape => KeyCode::Escape,
      keyboard::KeyCode::F1 => KeyCode::F1,
      keyboard::KeyCode::F2 => KeyCode::F2,
      keyboard::KeyCode::F3 => KeyCode::F3,
      keyboard::KeyCode::F4 => KeyCode::F4,
      keyboard::KeyCode::F5 => KeyCode::F5,
      keyboard::KeyCode::F6 => KeyCode::F6,
      keyboard::KeyCode::F7 => KeyCode::F7,
      keyboard::KeyCode::F8 => KeyCode::F8,
      keyboard::KeyCode::F9 => KeyCode::F9,
      keyboard::KeyCode::F10 => KeyCode::F10,
      keyboard::KeyCode::F11 => KeyCode::F11,
      keyboard::KeyCode::F12 => KeyCode::F12,
      keyboard::KeyCode::F13 => KeyCode::F13,
      keyboard::KeyCode::F14 => KeyCode::F14,
      keyboard::KeyCode::F15 => KeyCode::F15,
      keyboard::KeyCode::F16 => KeyCode::F16,
      keyboard::KeyCode::F17 => KeyCode::F17,
      keyboard::KeyCode::F18 => KeyCode::F18,
      keyboard::KeyCode::F19 => KeyCode::F19,
      keyboard::KeyCode::F20 => KeyCode::F20,
      keyboard::KeyCode::F21 => KeyCode::F21,
      keyboard::KeyCode::F22 => KeyCode::F22,
      keyboard::KeyCode::F23 => KeyCode::F23,
      keyboard::KeyCode::F24 => KeyCode::F24,
      keyboard::KeyCode::PrintScreen => KeyCode::PrintScreen,
      keyboard::KeyCode::ScrollLock => KeyCode::ScrollLock,
      keyboard::KeyCode::Pause => KeyCode::Pause,
      keyboard::KeyCode::Insert => KeyCode::Insert,
      keyboard::KeyCode::Home => KeyCode::Home,
      keyboard::KeyCode::Delete => KeyCode::Delete,
      keyboard::KeyCode::End => KeyCode::End,
      keyboard::KeyCode::PageDown => KeyCode::PageDown,
      keyboard::KeyCode::PageUp => KeyCode::PageUp,
      keyboard::KeyCode::ArrowLeft => KeyCode::ArrowLeft,
      keyboard::KeyCode::ArrowUp => KeyCode::ArrowUp,
      keyboard::KeyCode::ArrowRight => KeyCode::ArrowRight,
      keyboard::KeyCode::ArrowDown => KeyCode::ArrowDown,
      keyboard::KeyCode::Backspace => KeyCode::Backspace,
      keyboard::KeyCode::Enter => KeyCode::Enter,
      keyboard::KeyCode::Space => KeyCode::Space,
      keyboard::KeyCode::Numpad0 => KeyCode::Numpad0,
      keyboard::KeyCode::Numpad1 => KeyCode::Numpad1,
      keyboard::KeyCode::Numpad2 => KeyCode::Numpad2,
      keyboard::KeyCode::Numpad3 => KeyCode::Numpad3,
      keyboard::KeyCode::Numpad4 => KeyCode::Numpad4,
      keyboard::KeyCode::Numpad5 => KeyCode::Numpad5,
      keyboard::KeyCode::Numpad6 => KeyCode::Numpad6,
      keyboard::KeyCode::Numpad7 => KeyCode::Numpad7,
      keyboard::KeyCode::Numpad8 => KeyCode::Numpad8,
      keyboard::KeyCode::Numpad9 => KeyCode::Numpad9,
      keyboard::KeyCode::NumpadAdd => KeyCode::NumpadAdd,
      keyboard::KeyCode::NumpadDivide => KeyCode::NumpadDivide,
      keyboard::KeyCode::NumpadDecimal => KeyCode::NumpadDecimal,
      keyboard::KeyCode::NumpadComma => KeyCode::NumpadComma,
      keyboard::KeyCode::NumpadEnter => KeyCode::NumpadEnter,
      keyboard::KeyCode::NumpadMultiply => KeyCode::NumpadMultiply,
      keyboard::KeyCode::NumpadSubtract => KeyCode::NumpadSubtract,
      keyboard::KeyCode::Backslash => KeyCode::Backslash,
      keyboard::KeyCode::CapsLock => KeyCode::CapsLock,
      keyboard::KeyCode::Comma => KeyCode::Comma,
      keyboard::KeyCode::Convert => KeyCode::Convert,
      keyboard::KeyCode::MediaSelect => KeyCode::MediaSelect,
      keyboard::KeyCode::MediaStop => KeyCode::MediaStop,
      keyboard::KeyCode::Minus => KeyCode::Minus,
      keyboard::KeyCode::Period => KeyCode::Period,
      keyboard::KeyCode::Power => KeyCode::Power,
      keyboard::KeyCode::Semicolon => KeyCode::Semicolon,
      keyboard::KeyCode::Slash => KeyCode::Slash,
      keyboard::KeyCode::Sleep => KeyCode::Sleep,
      keyboard::KeyCode::Tab => KeyCode::Tab,
      keyboard::KeyCode::Copy => KeyCode::Copy,
      keyboard::KeyCode::Paste => KeyCode::Paste,
      keyboard::KeyCode::Cut => KeyCode::Cut,
      keyboard::KeyCode::Backquote => KeyCode::Backquote,
      keyboard::KeyCode::BracketLeft => KeyCode::BracketLeft,
      keyboard::KeyCode::BracketRight => KeyCode::BracketRight,
      keyboard::KeyCode::Equal => KeyCode::Equal,
      keyboard::KeyCode::IntlBackslash => KeyCode::IntlBackslash,
      keyboard::KeyCode::IntlRo => KeyCode::IntlRo,
      keyboard::KeyCode::IntlYen => KeyCode::IntlYen,
      keyboard::KeyCode::Quote => KeyCode::Quote,
      keyboard::KeyCode::AltLeft => KeyCode::AltLeft,
      keyboard::KeyCode::AltRight => KeyCode::AltRight,
      keyboard::KeyCode::ContextMenu => KeyCode::ContextMenu,
      keyboard::KeyCode::ControlLeft => KeyCode::ControlLeft,
      keyboard::KeyCode::ControlRight => KeyCode::ControlRight,
      keyboard::KeyCode::SuperLeft => KeyCode::SuperLeft,
      keyboard::KeyCode::SuperRight => KeyCode::SuperRight,
      keyboard::KeyCode::ShiftLeft => KeyCode::ShiftLeft,
      keyboard::KeyCode::ShiftRight => KeyCode::ShiftRight,
      keyboard::KeyCode::KanaMode => KeyCode::KanaMode,
      keyboard::KeyCode::Lang1 => KeyCode::Lang1,
      keyboard::KeyCode::Lang2 => KeyCode::Lang2,
      keyboard::KeyCode::Lang3 => KeyCode::Lang3,
      keyboard::KeyCode::Lang4 => KeyCode::Lang4,
      keyboard::KeyCode::Lang5 => KeyCode::Lang5,
      keyboard::KeyCode::NonConvert => KeyCode::NonConvert,
      keyboard::KeyCode::Help => KeyCode::Help,
      keyboard::KeyCode::NumLock => KeyCode::NumLock,
      keyboard::KeyCode::NumpadBackspace => KeyCode::NumpadBackspace,
      keyboard::KeyCode::NumpadClear => KeyCode::NumpadClear,
      keyboard::KeyCode::NumpadClearEntry => KeyCode::NumpadClearEntry,
      keyboard::KeyCode::NumpadEqual => KeyCode::NumpadEqual,
      keyboard::KeyCode::NumpadHash => KeyCode::NumpadHash,
      keyboard::KeyCode::NumpadMemoryAdd => KeyCode::NumpadMemoryAdd,
      keyboard::KeyCode::NumpadMemoryClear => KeyCode::NumpadMemoryClear,
      keyboard::KeyCode::NumpadMemoryRecall => KeyCode::NumpadMemoryRecall,
      keyboard::KeyCode::NumpadMemoryStore => KeyCode::NumpadMemoryStore,
      keyboard::KeyCode::NumpadMemorySubtract => KeyCode::NumpadMemorySubtract,
      keyboard::KeyCode::NumpadParenLeft => KeyCode::NumpadParenLeft,
      keyboard::KeyCode::NumpadParenRight => KeyCode::NumpadParenRight,
      keyboard::KeyCode::NumpadStar => KeyCode::NumpadStar,
      keyboard::KeyCode::Fn => KeyCode::Fn,
      keyboard::KeyCode::FnLock => KeyCode::FnLock,
      keyboard::KeyCode::BrowserBack => KeyCode::BrowserBack,
      keyboard::KeyCode::BrowserFavorites => KeyCode::BrowserFavorites,
      keyboard::KeyCode::BrowserForward => KeyCode::BrowserForward,
      keyboard::KeyCode::BrowserHome => KeyCode::BrowserHome,
      keyboard::KeyCode::BrowserRefresh => KeyCode::BrowserRefresh,
      keyboard::KeyCode::BrowserSearch => KeyCode::BrowserSearch,
      keyboard::KeyCode::BrowserStop => KeyCode::BrowserStop,
      keyboard::KeyCode::Eject => KeyCode::Eject,
      keyboard::KeyCode::LaunchApp1 => KeyCode::LaunchApp1,
      keyboard::KeyCode::LaunchApp2 => KeyCode::LaunchApp2,
      keyboard::KeyCode::LaunchMail => KeyCode::LaunchMail,
      keyboard::KeyCode::MediaPlayPause => KeyCode::MediaPlayPause,
      keyboard::KeyCode::MediaTrackNext => KeyCode::MediaTrackNext,
      keyboard::KeyCode::MediaTrackPrevious => KeyCode::MediaTrackPrevious,
      keyboard::KeyCode::AudioVolumeDown => KeyCode::AudioVolumeDown,
      keyboard::KeyCode::AudioVolumeMute => KeyCode::AudioVolumeMute,
      keyboard::KeyCode::AudioVolumeUp => KeyCode::AudioVolumeUp,
      keyboard::KeyCode::WakeUp => KeyCode::WakeUp,
      keyboard::KeyCode::Meta => KeyCode::Meta,
      keyboard::KeyCode::Hyper => KeyCode::Hyper,
      keyboard::KeyCode::Turbo => KeyCode::Turbo,
      keyboard::KeyCode::Abort => KeyCode::Abort,
      keyboard::KeyCode::Resume => KeyCode::Resume,
      keyboard::KeyCode::Suspend => KeyCode::Suspend,
      keyboard::KeyCode::Again => KeyCode::Again,
      keyboard::KeyCode::Find => KeyCode::Find,
      keyboard::KeyCode::Open => KeyCode::Open,
      keyboard::KeyCode::Props => KeyCode::Props,
      keyboard::KeyCode::Select => KeyCode::Select,
      keyboard::KeyCode::Undo => KeyCode::Undo,
      keyboard::KeyCode::Hiragana => KeyCode::Hiragana,
      keyboard::KeyCode::Katakana => KeyCode::Katakana,
      keyboard::KeyCode::F25 => KeyCode::F25,
      keyboard::KeyCode::F26 => KeyCode::F26,
      keyboard::KeyCode::F27 => KeyCode::F27,
      keyboard::KeyCode::F28 => KeyCode::F28,
      keyboard::KeyCode::F29 => KeyCode::F29,
      keyboard::KeyCode::F30 => KeyCode::F30,
      keyboard::KeyCode::F31 => KeyCode::F31,
      keyboard::KeyCode::F32 => KeyCode::F32,
      keyboard::KeyCode::F33 => KeyCode::F33,
      keyboard::KeyCode::F34 => KeyCode::F34,
      keyboard::KeyCode::F35 => KeyCode::F35,
      _ => KeyCode::Unknown,
    }
  }
}
