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
