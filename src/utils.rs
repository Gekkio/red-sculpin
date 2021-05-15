// SPDX-FileCopyrightText: 2020-2021 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Returns true if the given string is a valid program mnemonic.
///
/// Reference: IEEE 488.2: 7.6.1.2 - Encoding syntax
pub fn is_program_mnemonic(text: &str) -> bool {
    // ASCII alphabetic + 0-N times ASCII alphanumeric or underscore
    text.bytes().enumerate().any(|(idx, ch)| match ch {
        b'a'..=b'z' | b'A'..=b'Z' => true,
        b'0'..=b'9' if idx > 0 => true,
        b'_' if idx > 0 => true,
        _ => false,
    }) && !text.is_empty()
}
