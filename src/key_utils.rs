use crossterm::event::KeyCode;

/// Converts a [`KeyCode`] into a human readable string.
///
/// This helper is primarily used when serializing or displaying
/// [`crate::KeyBindings`].
///
/// # Arguments
/// * `key` - The key code to convert.
///
/// # Returns
/// A string representation of the key.
///
/// # Example
/// ```
/// use crossterm::event::KeyCode;
/// use real_time_note_taker::key_to_string;
/// assert_eq!(key_to_string(KeyCode::Enter), "Enter");
/// ```
///
/// # See also
/// [`string_to_key`]
#[must_use]
pub fn key_to_string(key: KeyCode) -> String {
    match key {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Null => "Null".to_string(),
        other => format!("{other:?}"),
    }
}

/// Parses a string previously produced by [`key_to_string`] back into a [`KeyCode`].
///
/// # Arguments
/// * `s` - The string slice to parse.
///
/// # Returns
/// The corresponding [`KeyCode`] if recognized, otherwise `None`.
///
/// # Example
/// ```
/// use crossterm::event::KeyCode;
/// use real_time_note_taker::string_to_key;
/// assert_eq!(string_to_key("Esc"), Some(KeyCode::Esc));
/// ```
///
/// # See also
/// [`key_to_string`]
#[must_use]
pub fn string_to_key(s: &str) -> Option<KeyCode> {
    match s {
        "Enter" => Some(KeyCode::Enter),
        "Esc" => Some(KeyCode::Esc),
        "Up" => Some(KeyCode::Up),
        "Down" => Some(KeyCode::Down),
        "Left" => Some(KeyCode::Left),
        "Right" => Some(KeyCode::Right),
        "Null" => Some(KeyCode::Null),
        c if c.len() == 1 => c.chars().next().map(KeyCode::Char),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_string_round_trip() {
        let keys = [
            KeyCode::Enter,
            KeyCode::Esc,
            KeyCode::Char('x'),
            KeyCode::Up,
            KeyCode::Null,
        ];
        for k in keys {
            let s = key_to_string(k);
            assert_eq!(string_to_key(&s), Some(k));
        }
    }

    #[test]
    fn string_to_key_invalid() {
        assert_eq!(string_to_key("invalid"), None);
    }

    #[test]
    fn key_to_string_unknown() {
        let out = key_to_string(KeyCode::F(1));
        assert!(!out.is_empty());
    }
}
