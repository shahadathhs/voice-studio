//! Italian sentences are understood when spoken. Ignored by default because it needs two models; see `speech_check/mod.rs`:
//!
//! ```text
//! VTT_TEST_KOKORO=<data dir>/models/kokoro-82m VTT_TEST_WHISPER=<data dir>/models/whisper-small/ggml-small.bin \
//!     cargo test -p vtt-core --release --test speech_italian_e2e -- --ignored --nocapture
//! ```
mod speech_check;

use speech_check::{Case, check};

const CASES: &[Case] = &[
    Case {
        voice: "if_sara",
        language: "it",
        text: "Buongiorno, mi chiamo Giulia e vivo a Roma da ventitré anni.",
        allowed: 0.25,
    },
    Case {
        voice: "im_nicola",
        language: "it",
        text: "Buongiorno, mi chiamo Giulia e vivo a Roma da ventitré anni.",
        allowed: 0.25,
    },
    Case {
        voice: "if_sara",
        language: "it",
        text: "Oggi ho comprato due chili di gnocchi, tre bottiglie d'acqua e il pane.",
        allowed: 0.25,
    },
    Case {
        voice: "im_nicola",
        language: "it",
        text: "Oggi ho comprato due chili di gnocchi, tre bottiglie d'acqua e il pane.",
        allowed: 0.25,
    },
    Case {
        voice: "if_sara",
        language: "it",
        text: "Il figlio di mia sorella studia la scienza e mangia il pesce con gli amici.",
        allowed: 0.05,
    },
    Case {
        voice: "im_nicola",
        language: "it",
        text: "Il figlio di mia sorella studia la scienza e mangia il pesce con gli amici.",
        allowed: 0.05,
    },
    Case {
        voice: "if_sara",
        language: "it",
        text: "Questa città è molto bella, e ci sono più di 2.500 persone nella piazza.",
        allowed: 0.05,
    },
    Case {
        voice: "im_nicola",
        language: "it",
        text: "Questa città è molto bella, e ci sono più di 2.500 persone nella piazza.",
        allowed: 0.05,
    },
    Case {
        voice: "if_sara",
        language: "it",
        text: "Vorrei un caffè e un gelato alla cena, per favore, perché ho molta fame.",
        allowed: 0.05,
    },
    Case {
        voice: "im_nicola",
        language: "it",
        text: "Vorrei un caffè e un gelato alla cena, per favore, perché ho molta fame.",
        allowed: 0.05,
    },
    Case {
        voice: "if_sara",
        language: "it",
        text: "Domani mattina partiamo presto per il mare, ma la macchina costa 1.250 euro.",
        allowed: 0.05,
    },
    Case {
        voice: "im_nicola",
        language: "it",
        text: "Domani mattina partiamo presto per il mare, ma la macchina costa 1.250 euro.",
        allowed: 0.05,
    },
    Case {
        voice: "if_sara",
        language: "it",
        text: "La famiglia ha una casa grande con un giardino, e la nonna cucina la pasta.",
        allowed: 0.05,
    },
    Case {
        voice: "im_nicola",
        language: "it",
        text: "La famiglia ha una casa grande con un giardino, e la nonna cucina la pasta.",
        allowed: 0.05,
    },
    Case {
        voice: "if_sara",
        language: "it",
        text: "Gli occhi della bottiglia brillano sulla tavola, mentre la famiglia mangia tagliatelle, pizza e spaghetti.",
        allowed: 0.05,
    },
    Case {
        voice: "im_nicola",
        language: "it",
        text: "Gli occhi della bottiglia brillano sulla tavola, mentre la famiglia mangia tagliatelle, pizza e spaghetti.",
        allowed: 0.05,
    },
    Case {
        voice: "if_sara",
        language: "it",
        text: "Il ragazzo con la faccia stanca lascia il bagno, poi aggiunge il cognome del signore sulla pagina, oggi.",
        allowed: 0.05,
    },
    Case {
        voice: "im_nicola",
        language: "it",
        text: "Il ragazzo con la faccia stanca lascia il bagno, poi aggiunge il cognome del signore sulla pagina, oggi.",
        allowed: 0.05,
    },
];

#[test]
#[ignore = "needs the Kokoro and Whisper models"]
fn spoken_sentences_are_understood_by_a_recogniser() {
    check(CASES);
}
