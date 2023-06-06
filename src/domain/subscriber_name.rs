use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(name: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = name.trim().is_empty();
        let is_to_long = name.graphemes(true).count() > 256;
        let forbidden_characters = ['/', '(', ')', '{', '}', '"', '<', '>', '\\'];
        let is_contains_forbidden_characters = name
            .chars()
            .any(|char| forbidden_characters.contains(&char));

        if is_empty_or_whitespace || is_to_long || is_contains_forbidden_characters {
            Err(format!("{} is not valid subscriber name", name))
        } else {
            Ok(Self(name))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_name_is_valid() {
        let name = "a".repeat(256);

        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_257_grapheme_name_is_rejected() {
        let name = "a".repeat(257);

        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_name_is_rejected() {
        let name = "      ".to_string();

        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_name_is_rejected() {
        let name = "".to_string();

        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn name_with_forbidden_characters_is_rejected() {
        for name in ['/', '(', ')', '{', '}', '"', '<', '>', '\\'] {
            assert_err!(SubscriberName::parse(name.to_string()));
        }
    }

    #[test]
    fn valid_name_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();

        assert_ok!(SubscriberName::parse(name));
    }
}
