//! The text conversions and key classification behind Windows consoles,
//! kept apart from the Windows calls so every platform's tests check them.

/// The length of `data` up to the end of its last complete UTF-8 sequence.
pub fn complete_prefix(data: &[u8]) -> usize {
    let tail = data.len().saturating_sub(3);
    for i in (tail..data.len()).rev() {
        let b = data[i];
        if b & 0xC0 != 0x80 {
            let need = match b {
                0x00..=0x7F => 1,
                0xC0..=0xDF => 2,
                0xE0..=0xEF => 3,
                0xF0..=0xF7 => 4,
                _ => 1,
            };
            return if data.len() - i < need { i } else { data.len() };
        }
    }
    data.len()
}

/// Converts UTF-16 units read from a console to UTF-8, carrying a high
/// surrogate that ends `units` over to the next read in `carry`, so a pair
/// split across reads is decoded whole. Ctrl-Z arrives as 0x1A, and an
/// unpaired surrogate as U+FFFD.
pub fn decode_units(carry: &mut Option<u16>, units: &[u16]) -> Vec<u8> {
    let mut all: Vec<u16> = carry.take().into_iter().collect();
    all.extend_from_slice(units);
    if let Some(&last) = all.last() {
        if (0xD800..0xDC00).contains(&last) {
            *carry = all.pop();
        }
    }
    String::from_utf16_lossy(&all).into_bytes()
}

/// The Alt key's virtual-key code, `VK_MENU`.
const VK_MENU: u16 = 0x12;

/// Whether a console key record carries a typed character: a key press
/// with one, or the Alt release that ends an Alt+Numpad entry (Alt+0233 is
/// `é`), which the console delivers on the release.
pub fn carries_input(key_down: bool, ch: u16, virtual_key: u16) -> bool {
    ch != 0 && (key_down || virtual_key == VK_MENU)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_press_or_an_alt_numpad_release_carries_input() {
        assert!(carries_input(true, u16::from(b'a'), 0x41));
        assert!(carries_input(false, 0xe9, VK_MENU));
        assert!(!carries_input(false, u16::from(b'a'), 0x41));
        assert!(!carries_input(true, 0, 0x10));
        assert!(!carries_input(false, 0, VK_MENU));
    }

    #[test]
    fn an_incomplete_utf8_sequence_is_held_back() {
        assert_eq!(complete_prefix(b"abc"), 3);
        let e_acute = "é".as_bytes();
        assert_eq!(complete_prefix(&[b'a', e_acute[0]]), 1);
        assert_eq!(complete_prefix(&[b'a', e_acute[0], e_acute[1]]), 3);
        let euro = "€".as_bytes();
        assert_eq!(complete_prefix(&euro[..2]), 0);
    }

    fn units(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn a_non_ascii_keystroke_and_ctrl_z_arrive_as_utf8_and_0x1a() {
        let mut carry = None;
        assert_eq!(
            decode_units(&mut carry, &units("é\u{1a}")),
            "é\u{1a}".as_bytes()
        );
        assert_eq!(carry, None);
    }

    #[test]
    fn an_arrow_key_arrives_as_its_escape_sequence() {
        let mut carry = None;
        assert_eq!(decode_units(&mut carry, &units("\u{1b}[A")), b"\x1b[A");
    }

    #[test]
    fn a_surrogate_pair_split_across_reads_is_decoded_whole() {
        let pair = units("😀");
        let mut carry = None;
        assert_eq!(decode_units(&mut carry, &[u16::from(b'a'), pair[0]]), b"a");
        assert_eq!(carry, Some(pair[0]));
        assert_eq!(decode_units(&mut carry, &[pair[1]]), "😀".as_bytes());
        assert_eq!(carry, None);
    }

    #[test]
    fn an_unpaired_low_surrogate_becomes_a_replacement_character() {
        let mut carry = None;
        assert_eq!(decode_units(&mut carry, &[0xDC00]), "\u{fffd}".as_bytes());
    }
}
