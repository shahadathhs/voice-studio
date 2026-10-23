//! Portuguese sentences are understood when spoken. Ignored by default because it needs two models; see `speech_check/mod.rs`:
//!
//! ```text
//! VTT_TEST_KOKORO=<data dir>/models/kokoro-82m VTT_TEST_WHISPER=<data dir>/models/whisper-small/ggml-small.bin \
//!     cargo test -p vtt-core --release --test speech_portuguese_e2e -- --ignored --nocapture
//! ```
mod speech_check;

use speech_check::{Case, check};

const CASES: &[Case] = &[
    Case {
        voice: "pf_dora",
        language: "pt",
        text: "Bom dia, eu quero um café com leite e pão.",
        allowed: 0.10,
    },
    Case {
        voice: "pf_dora",
        language: "pt",
        text: "A minha família mora numa casa grande perto do mar.",
        allowed: 0.10,
    },
    Case {
        voice: "pf_dora",
        language: "pt",
        text: "Hoje são 25 de dezembro de 2024.",
        allowed: 0.10,
    },
    Case {
        voice: "pf_dora",
        language: "pt",
        text: "O filho da vizinha trabalha com o irmão dele em Belo Horizonte.",
        allowed: 0.10,
    },
    Case {
        voice: "pf_dora",
        language: "pt",
        text: "Nós temos muitos amigos, e a gente gosta de ver o jogo de futebol.",
        allowed: 0.10,
    },
    Case {
        voice: "pf_dora",
        language: "pt",
        text: "A chuva caiu a noite inteira, mas amanhã o dia vai ser bonito.",
        allowed: 0.10,
    },
    Case {
        voice: "pf_dora",
        language: "pt",
        text: "O ônibus sai às 8 horas e a viagem dura 3 horas e meia.",
        allowed: 0.10,
    },
    Case {
        voice: "pf_dora",
        language: "pt",
        text: "Obrigado pela atenção, você fala português muito bem.",
        allowed: 0.10,
    },
    Case {
        voice: "pm_alex",
        language: "pt",
        text: "Bom dia, eu quero um café com leite e pão.",
        allowed: 0.10,
    },
    Case {
        voice: "pm_alex",
        language: "pt",
        text: "A minha família mora numa casa grande perto do mar.",
        allowed: 0.10,
    },
    Case {
        voice: "pm_alex",
        language: "pt",
        text: "Hoje são 25 de dezembro de 2024.",
        allowed: 0.10,
    },
    Case {
        voice: "pm_alex",
        language: "pt",
        text: "O filho da vizinha trabalha com o irmão dele em Belo Horizonte.",
        allowed: 0.10,
    },
    Case {
        voice: "pm_alex",
        language: "pt",
        text: "Nós temos muitos amigos, e a gente gosta de ver o jogo de futebol.",
        allowed: 0.10,
    },
    Case {
        voice: "pm_alex",
        language: "pt",
        text: "A chuva caiu a noite inteira, mas amanhã o dia vai ser bonito.",
        allowed: 0.10,
    },
    Case {
        voice: "pm_alex",
        language: "pt",
        text: "O ônibus sai às 8 horas e a viagem dura 3 horas e meia.",
        allowed: 0.10,
    },
    Case {
        voice: "pm_alex",
        language: "pt",
        text: "Obrigado pela atenção, você fala português muito bem.",
        allowed: 0.10,
    },
];

#[test]
#[ignore = "needs the Kokoro and Whisper models"]
fn spoken_sentences_are_understood_by_a_recogniser() {
    check(CASES);
}
