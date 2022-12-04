// SPDX-FileCopyrightText: 2020-2021 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Returns true if the given bytes form a valid program mnemonic.
///
/// Reference: IEEE 488.2: 7.6.1.2 - Encoding syntax
pub fn is_program_mnemonic(bytes: impl AsRef<[u8]>) -> bool {
    // ASCII alphabetic + 0-N times ASCII alphanumeric or underscore
    match bytes.as_ref() {
        [head, tail @ ..] => {
            head.is_ascii_alphabetic()
                && tail.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_')
        }
        [] => false,
    }
}

#[cfg(test)]
mod program_mnemonic {
    use crate::is_program_mnemonic;

    #[test]
    fn is_not_empty() {
        assert!(!is_program_mnemonic(&[]));
    }

    #[test]
    fn starts_with_ascii_alphabetic() {
        assert!(is_program_mnemonic("Hello"));
        assert!(is_program_mnemonic("hello"));
        assert!(!is_program_mnemonic("1fail"));
        assert!(!is_program_mnemonic("\nfail"));
    }

    #[test]
    fn contains_only_ascii_alphanumerics_and_underscores() {
        assert!(is_program_mnemonic("The_answer_IS_42"));
        assert!(!is_program_mnemonic("NOP€"));
        assert!(!is_program_mnemonic("NOPE!"));
        assert!(!is_program_mnemonic("NOPE\n"));
    }
}
