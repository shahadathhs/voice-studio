//! Chinese sentences are understood when spoken. Ignored by default because it needs two models; see
//! `speech_check/mod.rs`:
//!
//! ```text
//! VTT_TEST_KOKORO=<data dir>/models/kokoro-82m VTT_TEST_WHISPER=<data dir>/models/whisper-small/ggml-small.bin \
//!     cargo test -p vtt-core --release --test speech_chinese_e2e -- --ignored --nocapture
//! ```
mod speech_check;

use speech_check::{Case, check};

const CASES: &[Case] = &[
    Case {
        voice: "zf_xiaoxiao",
        language: "zh",
        text: "你好，今天天气很好。",
        allowed: 0.4,
    },
    Case {
        voice: "zf_xiaoxiao",
        language: "zh",
        text: "我喜欢学习中文。",
        allowed: 0.4,
    },
    Case {
        voice: "zm_yunxi",
        language: "zh",
        text: "北京是中国的首都。",
        allowed: 0.4,
    },
    Case {
        voice: "zm_yunxi",
        language: "zh",
        text: "明天我们一起去公园玩。",
        allowed: 0.4,
    },
    Case {
        voice: "zf_xiaobei",
        language: "zh",
        text: "这本书很有意思，我想再读一遍。",
        allowed: 0.4,
    },
    Case {
        voice: "zf_xiaoxiao",
        language: "zh",
        text: "他在银行工作，每天九点到公司。",
        allowed: 0.4,
    },
    Case {
        voice: "zm_yunjian",
        language: "zh",
        text: "我们有三个孩子，一共花了二十五块钱。",
        allowed: 0.4,
    },
    Case {
        voice: "zf_xiaoyi",
        language: "zh",
        text: "请问，火车站怎么走？",
        allowed: 0.4,
    },
    Case {
        voice: "zf_xiaoyi",
        language: "zh",
        text: "这个周末我想去看电影。",
        allowed: 0.4,
    },
    Case {
        voice: "zm_yunxia",
        language: "zh",
        text: "中文很有意思，我每天都学习。",
        allowed: 0.4,
    },
];

#[test]
#[ignore = "needs the Kokoro and Whisper models"]
fn spoken_sentences_are_understood_by_a_recogniser() {
    check(CASES);
}
