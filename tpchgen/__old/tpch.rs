mod constants {
    // List of container types from Section 4.2.2.13
    pub const CONTAINERS: [&str; 40] = [
        "SM CASE",
        "SM BOX",
        "SM BAG",
        "SM JAR",
        "SM PKG",
        "SM PACK",
        "SM CAN",
        "SM DRUM",
        "LG CASE",
        "LG BOX",
        "LG BAG",
        "LG JAR",
        "LG PKG",
        "LG PACK",
        "LG CAN",
        "LG DRUM",
        "MED CASE",
        "MED BOX",
        "MED BAG",
        "MED JAR",
        "MED PKG",
        "MED PACK",
        "MED CAN",
        "MED DRUM",
        "JUMBO CASE",
        "JUMBO BOX",
        "JUMBO BAG",
        "JUMBO JAR",
        "JUMBO PKG",
        "JUMBO PACK",
        "JUMBO CAN",
        "JUMBO DRUM",
        "WRAP CASE",
        "WRAP BOX",
        "WRAP BAG",
        "WRAP JAR",
        "WRAP PKG",
        "WRAP PACK",
        "WRAP CAN",
        "WRAP DRUM",
    ];

    // List of part types from Section 4.2.2.13
    pub const TYPES: [&str; 16] = [
        "STANDARD",
        "ANODIZED",
        "TIN",
        "SMALL",
        "BURNISHED",
        "NICKEL",
        "MEDIUM",
        "PLATED",
        "BRASS",
        "LARGE",
        "POLISHED",
        "STEEL",
        "ECONOMY",
        "BRUSHED",
        "COPPER",
        "PROMO",
    ];

    // Other word lists from the TPC-H spec
    pub const SEGMENTS: [&str; 5] = [
        "AUTOMOBILE",
        "BUILDING",
        "FURNITURE",
        "MACHINERY",
        "HOUSEHOLD",
    ];

    pub const PRIORITIES: [&str; 5] =
        ["1-URGENT", "2-HIGH", "3-MEDIUM", "4-NOT SPECIFIED", "5-LOW"];

    pub const INSTRUCTIONS: [&str; 4] = [
        "DELIVER IN PERSON",
        "COLLECT COD",
        "NONE",
        "TAKE BACK RETURN",
    ];

    pub const MODES: [&str; 7] = ["REG AIR", "AIR", "RAIL", "SHIP", "TRUCK", "MAIL", "FOB"];

    // Nation and region data
    pub const NATIONS: [(i32, &str, i32); 25] = [
        (0, "ALGERIA", 0),
        (1, "ARGENTINA", 1),
        (2, "BRAZIL", 1),
        (3, "CANADA", 1),
        (4, "EGYPT", 4),
        (5, "ETHIOPIA", 0),
        (6, "FRANCE", 3),
        (7, "GERMANY", 3),
        (8, "INDIA", 2),
        (9, "INDONESIA", 2),
        (10, "IRAN", 4),
        (11, "IRAQ", 4),
        (12, "JAPAN", 2),
        (13, "JORDAN", 4),
        (14, "KENYA", 0),
        (15, "MOROCCO", 0),
        (16, "MOZAMBIQUE", 0),
        (17, "PERU", 1),
        (18, "CHINA", 2),
        (19, "ROMANIA", 3),
        (20, "SAUDI ARABIA", 4),
        (21, "VIETNAM", 2),
        (22, "RUSSIA", 3),
        (23, "UNITED KINGDOM", 3),
        (24, "UNITED STATES", 1),
    ];

    pub const REGIONS: [(i32, &str); 5] = [
        (0, "AFRICA"),
        (1, "AMERICA"),
        (2, "ASIA"),
        (3, "EUROPE"),
        (4, "MIDDLE EAST"),
    ];

    // Date constants from Section 4.2.2.12
    pub const START_DATE: &str = "1992-01-01";
    pub const CURRENT_DATE: &str = "1995-06-17";
    pub const END_DATE: &str = "1998-12-31";

    pub const NOUNS: [&str; 41] = [
        "foxes",
        "ideas",
        "theodolites",
        "pinto beans",
        "instructions",
        "dependencies",
        "excuses",
        "platelets",
        "asymptotes",
        "courts",
        "dolphins",
        "multipliers",
        "sauternes",
        "warthogs",
        "frets",
        "dinos",
        "attainments",
        "somas",
        "Tiresias'",
        "patterns",
        "forges",
        "braids",
        "hockey players",
        "frays",
        "warhorses",
        "dugouts",
        "notornis",
        "epitaphs",
        "pearls",
        "tithes",
        "waters",
        "orbits",
        "gifts",
        "sheaves",
        "depths",
        "sentiments",
        "decoys",
        "realms",
        "pains",
        "grouches",
        "escapades",
    ];

    pub const VERBS: [&str; 40] = [
        "sleep",
        "wake",
        "are",
        "cajole",
        "haggle",
        "nag",
        "use",
        "boost",
        "affix",
        "detect",
        "integrate",
        "maintain",
        "nod",
        "was",
        "lose",
        "sublate",
        "solve",
        "thrash",
        "promise",
        "engage",
        "hinder",
        "print",
        "x-ray",
        "breach",
        "eat",
        "grow",
        "impress",
        "mold",
        "poach",
        "serve",
        "run",
        "dazzle",
        "snooze",
        "doze",
        "unwind",
        "kindle",
        "play",
        "hang",
        "believe",
        "doubt",
    ];

    pub const ADJECTIVES: [&str; 25] = [
        "furious",
        "sly",
        "careful",
        "blithe",
        "quick",
        "fluffy",
        "slow",
        "quiet",
        "ruthless",
        "thin",
        "close",
        "dogged",
        "daring",
        "brave",
        "stealthy",
        "permanent",
        "enticing",
        "idle",
        "busy",
        "regular",
        "final",
        "ironic",
        "even",
        "bold",
        "silent",
    ];

    pub const ADVERBS: [&str; 28] = [
        "sometimes",
        "always",
        "never",
        "furiously",
        "slyly",
        "carefully",
        "blithely",
        "quickly",
        "fluffily",
        "slowly",
        "quietly",
        "ruthlessly",
        "thinly",
        "closely",
        "doggedly",
        "daringly",
        "bravely",
        "stealthily",
        "permanently",
        "enticingly",
        "idly",
        "busily",
        "regularly",
        "finally",
        "ironically",
        "evenly",
        "boldly",
        "silently",
    ];

    pub const PREPOSITIONS: [&str; 47] = [
        "about",
        "above",
        "according to",
        "across",
        "after",
        "against",
        "along",
        "alongside of",
        "among",
        "around",
        "at",
        "atop",
        "before",
        "behind",
        "beneath",
        "beside",
        "besides",
        "between",
        "beyond",
        "by",
        "despite",
        "during",
        "except",
        "for",
        "from",
        "in place of",
        "inside",
        "instead of",
        "into",
        "near",
        "of",
        "on",
        "outside",
        "over",
        "past",
        "since",
        "through",
        "throughout",
        "to",
        "toward",
        "under",
        "until",
        "up",
        "upon",
        "without",
        "with",
        "within",
    ];

    pub const AUXILIARIES: [&str; 18] = [
        "do",
        "may",
        "might",
        "shall",
        "will",
        "would",
        "can",
        "could",
        "should",
        "ought to",
        "must",
        "will have to",
        "shall have to",
        "could have to",
        "should have to",
        "must have to",
        "need to",
        "try to",
    ];

    pub const TERMINATORS: [&str; 6] = [".", ";", ":", "?", "!", "--"];
}

/// Scale factors used to specify the size of the dataset to generate
/// a scale factor of 1 is approximately 1GB and is considered the smallest
/// one.
#[derive(Default, Debug, Clone, Copy)]
pub struct ScaleFactor(usize);

impl std::fmt::Display for ScaleFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            1 => write!(f, "1 GB"),
            10 => write!(f, "10 GB"),
            30 => write!(f, "30 GB"),
            100 => write!(f, "100 GB"),
            300 => write!(f, "300 GB"),
            1000 => write!(f, "1000 GB"),
            3000 => write!(f, "3000 GB"),
            10000 => write!(f, "10000 GB"),
            _ => write!(f, "{}", self.0),
        }
    }
}

impl ScaleFactor {
    /// Creates a new scale factor value and it must be in the set of allowed
    /// factors by the spec (1, 10, 30, 300, 1000, 3000, 10000, 30000, 100000).
    pub fn new(sf: usize) -> Self {
        debug_assert!(matches!(
            sf,
            1 | 10 | 30 | 100 | 300 | 1000 | 3000 | 10000 | 30000 | 100000
        ));

        Self(sf)
    }
}
