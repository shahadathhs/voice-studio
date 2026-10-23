//! Hindi sentences are understood when spoken. Ignored by default because it needs two models; see `speech_check/mod.rs`:
//!
//! ```text
//! VTT_TEST_KOKORO=<data dir>/models/kokoro-82m VTT_TEST_WHISPER=<data dir>/models/whisper-small/ggml-small.bin \
//!     cargo test -p vtt-core --release --test speech_hindi_e2e -- --ignored --nocapture
//! ```
mod speech_check;

use speech_check::{Case, check};

const CASES: &[Case] = &[
    Case {
        voice: "hf_alpha",
        language: "hi",
        text: "मैं आज बाज़ार जा रहा हूँ।",
        allowed: 0.3,
    },
    Case {
        voice: "hm_omega",
        language: "hi",
        text: "हमारा देश बहुत सुंदर है।",
        allowed: 0.3,
    },
    Case {
        voice: "hf_alpha",
        language: "hi",
        text: "आपका नाम क्या है? मेरा नाम प्रिया है।",
        allowed: 0.3,
    },
    Case {
        voice: "hm_omega",
        language: "hi",
        text: "कल सुबह हम स्कूल जाएँगे और किताबें पढ़ेंगे।",
        allowed: 0.3,
    },
    Case {
        voice: "hf_alpha",
        language: "hi",
        text: "मुझे चाय और गरम रोटी बहुत पसंद है।",
        allowed: 0.3,
    },
    Case {
        voice: "hm_omega",
        language: "hi",
        text: "भारत की राजधानी नई दिल्ली है।",
        allowed: 0.3,
    },
    Case {
        voice: "hf_alpha",
        language: "hi",
        text: "आज मौसम बहुत अच्छा है, चलो बाहर घूमने चलते हैं।",
        allowed: 0.3,
    },
    Case {
        voice: "hm_omega",
        language: "hi",
        text: "क्या आप मुझे सही रास्ता बता सकते हैं?",
        allowed: 0.3,
    },
    Case {
        voice: "hf_alpha",
        language: "hi",
        text: "मेरे पास पच्चीस रुपये हैं।",
        allowed: 0.5,
    },
    Case {
        voice: "hm_omega",
        language: "hi",
        text: "इस किताब में 150 पन्ने हैं और इसकी कीमत 2500 रुपये है।",
        allowed: 0.5,
    },
];

#[test]
#[ignore = "needs the Kokoro and Whisper models"]
fn spoken_sentences_are_understood_by_a_recogniser() {
    check(CASES);
}
