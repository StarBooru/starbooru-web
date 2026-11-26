//! Narrowing a page's list by the words typed in the search box, as Upload
//! and Settings do: everything that holds every word, in any case, with or
//! without accents.

/// The words of `q`, folded, in the order typed.
pub fn words(q: &str) -> Vec<String> {
    q.split_whitespace().map(fold).collect()
}

/// `s` in lower case with its accents taken off, so `perche` finds `Perché`.
pub fn fold(s: &str) -> String {
    s.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => 'a',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            other => other,
        })
        .collect()
}

/// Whether the texts in `places`, together, hold every one of `words`.
/// No words: everything does.
pub fn holds_all<S: AsRef<str>>(words: &[String], places: &[S]) -> bool {
    if words.is_empty() {
        return true;
    }
    let all = places
        .iter()
        .map(|p| fold(p.as_ref()))
        .collect::<Vec<_>>()
        .join("\n");
    words.iter().all(|w| all.contains(w.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_are_folded_and_split_on_spaces() {
        assert_eq!(words("  Blue   SKY "), ["blue", "sky"]);
        assert!(words("   ").is_empty());
        assert_eq!(fold("Perché Così ÀÉ"), "perche cosi ae");
    }

    #[test]
    fn every_word_has_to_be_somewhere() {
        let places = ["Holiday.JPG", "sea sunset"];
        assert!(holds_all(&words("holi"), &places));
        assert!(holds_all(&words("sunset jpg"), &places));
        assert!(!holds_all(&words("sunset cat"), &places));
        assert!(holds_all(&words(""), &places));
        // A word doesn't run across two places.
        assert!(!holds_all(&words("jpgsea"), &places));
        assert!(holds_all(&words("perche"), &["Perché no"]));
        assert!(holds_all(&words("PERCHÉ"), &["perche"]));
    }
}
