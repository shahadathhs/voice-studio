//! French sentences are understood when spoken. Ignored by default because it needs two models; see `speech_check/mod.rs`:
//!
//! ```text
//! VTT_TEST_KOKORO=<data dir>/models/kokoro-82m VTT_TEST_WHISPER=<data dir>/models/whisper-small/ggml-small.bin \
//!     cargo test -p vtt-core --release --test speech_french_e2e -- --ignored --nocapture
//! ```
mod speech_check;

use speech_check::{Case, check};

const CASES: &[Case] = &[
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "Bonjour, je m'appelle Marie et j'habite à Paris.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "Nous avons mangé du pain avec du fromage ce matin.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "L'été dernier, nous sommes allés à la montagne.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "Il y a 22 livres sur la table.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "Les enfants jouent dans le jardin pendant que la pluie tombe.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "Elle a acheté 340 pommes pour 90 euros.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "Je voudrais un café et un croissant, s'il vous plaît.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "Mon frère travaille dans une grande entreprise à Lyon.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "La nuit était belle et tout le monde regardait les étoiles.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "Il fait très chaud aujourd'hui, mais demain il pleuvra.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "Cette année, la nation fête son anniversaire en septembre.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "Je ne sais pas si vingt personnes viendront dimanche.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "Ma grand-mère prépare une soupe délicieuse pour toute la famille.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "Le directeur a expliqué la situation aux journalistes hier soir.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "Il y a 3,5 mètres entre la gare et notre maison.",
        allowed: 0.15,
    },
    Case {
        voice: "ff_siwis",
        language: "fr",
        text: "Elle dit qu'elle n'aime pas le poisson, mais elle adore les fruits.",
        allowed: 0.15,
    },
];

#[test]
#[ignore = "needs the Kokoro and Whisper models"]
fn spoken_sentences_are_understood_by_a_recogniser() {
    check(CASES);
}
