export interface BuildStep {
  title: string;
  body: string;
  code?: string;
}

/** Keep in step with docs/building.md. */
export const PREREQUISITES = [
  "Rust (stable) from rustup.rs",
  "A C++ compiler and CMake (whisper.cpp is compiled during the build)",
  "Node.js 22 or newer, and pnpm",
  "make (on Windows, Git Bash or WSL)",
  "Linux only: the Tauri system packages and libasound2-dev for the microphone",
];

export const BUILD_STEPS: BuildStep[] = [
  {
    title: "Get the code and set up",
    body: "Setup installs Rust and CMake if they are missing, the JavaScript dependencies and the git hooks.",
    code: `git clone https://github.com/shahadathhs/voice-studio.git
cd voice-studio
make setup`,
  },
  {
    title: "Build",
    body: "This builds the command-line tool and the desktop app in release mode. The first build compiles whisper.cpp and takes a while.",
    code: "make build",
  },
  {
    title: "Install",
    body: "The app is in target/release/bundle. On macOS copy Voice Studio.app to /Applications; on Windows and Linux run the installer or package you built. The CLI is target/release/vtt.",
    code: `# macOS
cp -R "target/release/bundle/macos/Voice Studio.app" /Applications/

# the command-line tool, on your PATH
cargo install --path crates/cli --locked`,
  },
];
