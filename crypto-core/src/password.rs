use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::error::{CryptoError, Result};

/// Character sets for password generation
const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";
const AMBIGUOUS: &[u8] = b"0O1lI";

/// Options for password generation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PasswordOptions {
    /// Length of the generated password
    pub length: usize,
    /// Include lowercase letters
    pub lowercase: bool,
    /// Include uppercase letters
    pub uppercase: bool,
    /// Include digits
    pub digits: bool,
    /// Include symbols
    pub symbols: bool,
    /// Exclude ambiguous characters (0, O, 1, l, I)
    pub exclude_ambiguous: bool,
    /// Custom characters to exclude
    pub exclude_chars: String,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            length: 16,
            lowercase: true,
            uppercase: true,
            digits: true,
            symbols: true,
            exclude_ambiguous: false,
            exclude_chars: String::new(),
        }
    }
}

impl PasswordOptions {
    pub fn new(length: usize) -> Self {
        Self {
            length,
            ..Default::default()
        }
    }

    pub fn with_lowercase(mut self, enabled: bool) -> Self {
        self.lowercase = enabled;
        self
    }

    pub fn with_uppercase(mut self, enabled: bool) -> Self {
        self.uppercase = enabled;
        self
    }

    pub fn with_digits(mut self, enabled: bool) -> Self {
        self.digits = enabled;
        self
    }

    pub fn with_symbols(mut self, enabled: bool) -> Self {
        self.symbols = enabled;
        self
    }

    pub fn with_exclude_ambiguous(mut self, enabled: bool) -> Self {
        self.exclude_ambiguous = enabled;
        self
    }

    pub fn with_exclude_chars(mut self, chars: &str) -> Self {
        self.exclude_chars = chars.to_string();
        self
    }
}

/// Generate a random password based on the given options
pub fn generate_password(options: &PasswordOptions) -> Result<String> {
    if options.length == 0 {
        return Err(CryptoError::InvalidPasswordOptions(
            "Password length must be at least 1".to_string(),
        ));
    }

    if options.length > 1024 {
        return Err(CryptoError::InvalidPasswordOptions(
            "Password length must not exceed 1024".to_string(),
        ));
    }

    // Build character pool
    let mut pool: Vec<u8> = Vec::new();
    let mut required_chars: Vec<u8> = Vec::new();

    let exclude_set: std::collections::HashSet<u8> = options
        .exclude_chars
        .bytes()
        .chain(if options.exclude_ambiguous {
            AMBIGUOUS.iter().copied()
        } else {
            [].iter().copied()
        })
        .collect();

    let filter_chars = |chars: &[u8]| -> Vec<u8> {
        chars
            .iter()
            .copied()
            .filter(|c| !exclude_set.contains(c))
            .collect()
    };

    if options.lowercase {
        let chars = filter_chars(LOWERCASE);
        if !chars.is_empty() {
            required_chars.push(*chars.first().unwrap());
            pool.extend(chars);
        }
    }

    if options.uppercase {
        let chars = filter_chars(UPPERCASE);
        if !chars.is_empty() {
            required_chars.push(*chars.first().unwrap());
            pool.extend(chars);
        }
    }

    if options.digits {
        let chars = filter_chars(DIGITS);
        if !chars.is_empty() {
            required_chars.push(*chars.first().unwrap());
            pool.extend(chars);
        }
    }

    if options.symbols {
        let chars = filter_chars(SYMBOLS);
        if !chars.is_empty() {
            required_chars.push(*chars.first().unwrap());
            pool.extend(chars);
        }
    }

    if pool.is_empty() {
        return Err(CryptoError::InvalidPasswordOptions(
            "At least one character type must be enabled".to_string(),
        ));
    }

    let mut rng = rand::thread_rng();

    // Generate password ensuring at least one character from each enabled type
    let mut password: Vec<u8> = Vec::with_capacity(options.length);

    // First, add required characters (one from each enabled type)
    for c in required_chars.iter().take(options.length) {
        password.push(*c);
    }

    // Fill the rest with random characters from the pool
    while password.len() < options.length {
        let idx = rng.gen_range(0..pool.len());
        password.push(pool[idx]);
    }

    // Shuffle to randomize positions
    password.shuffle(&mut rng);

    String::from_utf8(password).map_err(|e| CryptoError::InvalidPasswordOptions(e.to_string()))
}

/// Generate a passphrase using random words
pub fn generate_passphrase(word_count: usize, separator: &str) -> Result<String> {
    if word_count == 0 {
        return Err(CryptoError::InvalidPasswordOptions(
            "Word count must be at least 1".to_string(),
        ));
    }

    if word_count > 20 {
        return Err(CryptoError::InvalidPasswordOptions(
            "Word count must not exceed 20".to_string(),
        ));
    }

    // EFF word list (abbreviated for size - in production use full list)
    const WORDS: &[&str] = &[
        "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract", "absurd",
        "abuse", "access", "accident", "account", "accuse", "achieve", "acid", "acoustic",
        "acquire", "across", "action", "actor", "actress", "actual", "adapt", "address", "adjust",
        "admit", "adult", "advance", "advice", "aerobic", "affair", "afford", "afraid", "again",
        "age", "agent", "agree", "ahead", "aim", "air", "airport", "aisle", "alarm", "album",
        "alcohol", "alert", "alien", "allow", "almost", "alone", "alpha", "already", "also",
        "alter", "always", "amateur", "amazing", "among", "amount", "amused", "analyst", "anchor",
        "ancient", "anger", "angle", "angry", "animal", "ankle", "announce", "annual", "another",
        "answer", "antenna", "antique", "anxiety", "apart", "apology", "appear", "apple",
        "approve", "april", "arch", "arctic", "area", "arena", "argue", "arm", "armed", "armor",
        "army", "around", "arrange", "arrest", "arrive", "arrow", "art", "artist", "artwork",
        "aspect", "assault", "asset", "assist", "assume", "asthma", "athlete", "atom", "attack",
        "attend", "attract", "auction", "audit", "august", "aunt", "author", "auto", "autumn",
        "average", "avocado", "avoid", "awake", "aware", "away", "awesome", "awful", "awkward",
        "axis", "baby", "bachelor", "bacon", "badge", "bag", "balance", "balcony", "ball",
        "bamboo", "banana", "banner", "basket", "battle", "beach", "beauty", "become", "bedroom",
        "before", "begin", "believe", "below", "bench", "benefit", "best", "better", "between",
        "beyond", "bicycle", "bird", "birth", "bitter", "black", "blade", "blame", "blanket",
        "blast", "bleak", "bless", "blind", "blood", "blossom", "blouse", "blue", "board", "boat",
        "body", "boil", "bomb", "bone", "bonus", "book", "boost", "border", "boring", "borrow",
        "boss", "bottom", "bounce", "box", "brain", "brand", "brave", "bread", "breeze", "brick",
        "bridge", "brief", "bright", "bring", "broken", "bronze", "brother", "brown", "brush",
        "bubble", "bucket", "budget", "buffalo", "build", "bulb", "bulk", "bullet", "bundle",
        "burden", "burger", "burst", "butter", "cabin", "cable", "cactus", "cage", "camera",
        "camp", "canal", "cancel", "candy", "cannon", "canyon", "capable", "capital", "captain",
        "carbon", "career", "cargo", "carpet", "carry", "cart", "castle", "casual", "catalog",
        "catch", "category", "cattle", "ceiling", "celery", "cement", "census", "century",
        "cereal", "certain", "chair", "chalk", "champion", "change", "chaos", "chapter", "charge",
        "charity", "cheap", "cheese", "cherry", "chicken", "chief", "child", "choice", "chunk",
        "churn", "circle", "citizen", "city", "civil", "claim", "clap", "clarify", "claw", "clay",
        "clean", "clerk", "clever", "click", "client", "cliff", "climb", "clinic", "clip", "clock",
        "close", "cloth", "cloud", "clown", "club", "cluster", "coach", "coast", "coconut", "code",
        "coffee", "coin", "collect", "color", "column", "combine", "comfort", "comic", "common",
        "company", "concert", "conduct", "confirm", "congress", "connect", "consider", "control",
        "convince", "cookie", "copper", "coral", "corner", "correct", "couch", "country", "couple",
        "course", "cousin", "cover", "coyote", "crack", "cradle", "craft", "crane", "crash",
        "crater", "crazy", "cream", "credit", "creek", "crew", "cricket", "crime", "crisp",
        "critic", "crop", "cross", "crouch", "crowd", "crucial", "cruel", "cruise", "crumble",
        "crush", "crystal", "cube", "culture", "cupboard", "curious", "current", "curtain",
        "curve", "cushion", "custom", "cycle", "damage", "dance", "danger", "daring", "dash",
        "daughter", "dawn", "decade", "decide", "decline", "decorate", "decrease", "deep",
        "defense", "define", "delay", "deliver", "demand", "denial", "dentist", "deny", "depart",
        "depend", "deposit", "depth", "deputy", "derive", "describe", "desert", "design", "desk",
        "despair", "destroy", "detail", "detect", "develop", "device", "devote", "diagram",
        "diamond", "diary", "diesel", "diet", "differ", "digital", "dignity", "dilemma", "dinner",
        "dinosaur", "direct", "dirt", "disagree", "discover", "disease", "dish", "dismiss",
        "display", "distance", "divert", "divide", "divorce", "dizzy", "doctor", "document",
        "domain", "donate", "donkey", "door", "dose", "double", "dove", "draft", "dragon", "drama",
        "drastic", "draw", "dream", "dress", "drift", "drill", "drink", "drip", "drive", "drop",
        "drum", "dry", "duck", "dumb", "dune", "during", "dust", "dutch", "duty", "dwarf",
        "dynamic", "eager", "eagle", "early", "earth", "easily", "east", "easy", "echo", "ecology",
        "economy", "edge", "edit", "educate", "effort", "eight", "either", "elbow", "elder",
        "electric", "elegant", "element", "elephant", "elevator", "elite", "else", "embark",
        "embody", "embrace", "emerge", "emotion", "employ", "empower", "empty", "enable", "enact",
        "endless", "endorse", "enemy", "energy", "enforce", "engage", "engine", "enhance", "enjoy",
        "enlist", "enough", "enrich", "enroll",
    ];

    let mut rng = rand::thread_rng();
    let words: Vec<&str> = (0..word_count)
        .map(|_| {
            let idx = rng.gen_range(0..WORDS.len());
            WORDS[idx]
        })
        .collect();

    Ok(words.join(separator))
}

/// Calculate password entropy in bits
pub fn calculate_entropy(options: &PasswordOptions) -> f64 {
    let mut pool_size = 0;

    if options.lowercase {
        pool_size += 26;
    }
    if options.uppercase {
        pool_size += 26;
    }
    if options.digits {
        pool_size += 10;
    }
    if options.symbols {
        pool_size += SYMBOLS.len();
    }

    if options.exclude_ambiguous {
        // Remove ambiguous characters from count
        let ambiguous_count = AMBIGUOUS.len();
        pool_size = pool_size.saturating_sub(ambiguous_count);
    }

    if pool_size == 0 {
        return 0.0;
    }

    options.length as f64 * (pool_size as f64).log2()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_password_default() {
        let options = PasswordOptions::default();
        let password = generate_password(&options).unwrap();

        assert_eq!(password.len(), 16);
        assert!(password.chars().any(|c| c.is_lowercase()));
        assert!(password.chars().any(|c| c.is_uppercase()));
        assert!(password.chars().any(|c| c.is_numeric()));
    }

    #[test]
    fn test_generate_password_custom_length() {
        let options = PasswordOptions::new(32);
        let password = generate_password(&options).unwrap();

        assert_eq!(password.len(), 32);
    }

    #[test]
    fn test_generate_password_lowercase_only() {
        let options = PasswordOptions::new(20)
            .with_lowercase(true)
            .with_uppercase(false)
            .with_digits(false)
            .with_symbols(false);

        let password = generate_password(&options).unwrap();

        assert_eq!(password.len(), 20);
        assert!(password.chars().all(|c| c.is_lowercase()));
    }

    #[test]
    fn test_generate_password_exclude_ambiguous() {
        let options = PasswordOptions::new(100).with_exclude_ambiguous(true);

        for _ in 0..10 {
            let password = generate_password(&options).unwrap();
            assert!(!password.contains('0'));
            assert!(!password.contains('O'));
            assert!(!password.contains('1'));
            assert!(!password.contains('l'));
            assert!(!password.contains('I'));
        }
    }

    #[test]
    fn test_generate_password_exclude_chars() {
        let options = PasswordOptions::new(100)
            .with_symbols(false)
            .with_exclude_chars("aeiou");

        for _ in 0..10 {
            let password = generate_password(&options).unwrap();
            assert!(!password.contains('a'));
            assert!(!password.contains('e'));
            assert!(!password.contains('i'));
            assert!(!password.contains('o'));
            assert!(!password.contains('u'));
        }
    }

    #[test]
    fn test_generate_password_invalid_length() {
        let options = PasswordOptions::new(0);
        assert!(generate_password(&options).is_err());
    }

    #[test]
    fn test_generate_password_no_character_types() {
        let options = PasswordOptions::new(16)
            .with_lowercase(false)
            .with_uppercase(false)
            .with_digits(false)
            .with_symbols(false);

        assert!(generate_password(&options).is_err());
    }

    #[test]
    fn test_generate_passphrase() {
        let passphrase = generate_passphrase(4, "-").unwrap();
        let words: Vec<&str> = passphrase.split('-').collect();

        assert_eq!(words.len(), 4);
        assert!(words.iter().all(|w| !w.is_empty()));
    }

    #[test]
    fn test_calculate_entropy() {
        let options = PasswordOptions::new(16);
        let entropy = calculate_entropy(&options);

        // 16 chars from pool of ~90 chars ≈ 104 bits
        assert!(entropy > 100.0);
        assert!(entropy < 110.0);
    }
}
