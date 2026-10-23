/** Voice cloning: a voice made from a recording of someone who has given permission. */
export const clone = {
  "clone.title": "Clone a voice",
  "clone.hint":
    "Make a voice from 3 to 30 seconds of clear speech by one person. It runs on this computer. Clone only a voice you own or have permission to use.",
  "clone.needModel": "Cloning needs the Chatterbox model (about 1.5 GB).",
  "clone.openModels": "Get the model",
  "clone.file": "Recording",
  "clone.chooseFile": "Choose a recording",
  "clone.noFile": "No recording chosen",
  "clone.name": "Name",
  "clone.namePlaceholder": "Anna",
  "clone.consent": "I own this voice or have the speaker's permission",
  "clone.make": "Clone voice",
  "clone.working": "Cloning the voice. The first time loads a large model and can take a minute.",
  "clone.disclosure":
    "Audio made with a cloned voice is saved with a note that it is synthetic. It is not watermarked.",
  "clone.needAll": "Choose a recording, give the voice a name and confirm permission.",
  "clone.group": "Cloned voices",
  "clone.badge": "cloned",
  "clone.delete": "Delete the cloned voice {name}",
  "clone.language": "Language of the text",
  "clone.slow": "Cloned voices are slower than the built-in ones.",
} as const;
