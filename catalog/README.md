# Model catalog

`models.json` lists the models the app can download. It is compiled into the binary.

Each entry has `id`, `name`, `kind` (`stt` | `tts` | `diarization` | `vad`), `description`, `languages`,
`license`, and a list of `files`. Each file needs a real `name`, `url`, `sha256` and `size_bytes`. Downloads
are verified against the checksum and the size before a model counts as installed.

Rules for adding a model:

- Take the URL, size and checksum from the author's actual release, not from memory. For Hugging Face
  repositories, `https://huggingface.co/api/models/<repo>/tree/main` lists each file's `lfs.oid`
  (the SHA-256) and `size`.
- Pin the URL to a commit (`/resolve/<commit>/<file>`), never `main`, so the file cannot change under us.
- Check the model's licence. Only permissively licensed models belong here (MIT, Apache-2.0, BSD).

The speech-to-text entries are the ggml conversions of OpenAI Whisper published in
[ggerganov/whisper.cpp](https://huggingface.co/ggerganov/whisper.cpp) (MIT).

The speaker entry `speaker-identification` installs two models. The voice model is the ONNX export published by the
WeSpeaker authors
([Wespeaker/wespeaker-voxceleb-resnet34](https://huggingface.co/Wespeaker/wespeaker-voxceleb-resnet34)). Its model
card says Apache-2.0; WeSpeaker's documentation says VoxCeleb-trained models follow the dataset licence
(CC BY 4.0). The turn model is pyannote segmentation-3.0 (MIT, sherpa-onnx's ONNX export). Both permit
commercial use, but the dataset terms have not been reviewed. See
`docs/product/diarization.md`.

`can_translate` (default `true`) is `false` for the large-v3 turbo models, which Whisper trained without
translation.

The text-to-speech entry `kokoro-82m` is the 8-bit ONNX export of Kokoro-82M by onnx-community (Apache-2.0), with the
`misaki` pronunciation dictionaries (Apache-2.0, pinned to a commit of github.com/hexgrad/misaki) and six voices. The
file names must match the constants in `crates/core/src/tts/kokoro.rs`; a test checks it. See
`docs/product/speech.md`.
