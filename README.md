<div align="center">

# Pincer Chat

<img src="assets/logo.svg" width="200px"/>

<em>PincerChat, a desktop GUI for interacting with local LLMs served with <a href=https://ollama.com/>Ollama</a></em>

<img src="screenshot.png" width="800px"/>

</div>

***Note:*** *This project is very much a work-in-progress with an uncertain development because I am busy with other things. I just decided to open-source it now because it's in a good enough state and because it could be useful to anyone who stumbles upon it.*

## Features

- **Simple and responsive UI**: A clean interface for interacting with LLMs built with GTK4.
- **Response streaming**: Receive LLM responses as they're being generated.
- **Chat history**: Track previous interactions for reference.


## Roadmap

The following features are planned for future updates:

- [ ] **Markdown rendering** for rich text display.
- [ ] **File uploads**: Ability to upload and interact with files and documents.
- [ ] **Cross-platform executables**: Compiled Linux, macOS, and Windows executables for easier installation.

## Why another LLM frontend ?

When I first got into Rust development at the end of 2024, I wanted to tackle a project that would push my limits as a software engineer. Since I already had experience with LLMs, I decided to to build a desktop GUI for Ollama that could serve as both a challenge (since it's something I haven't built before) and a useful tool.

The idea was simple: create a clean, user-friendly interface for interacting with LLMs without needing a web browser. What I didn't realize was just how much I would learn along the way. From managing complex state interactions to navigating the intricacies of frontend design, this project has been both humbling and rewarding. It has deepened my appreciation for the art of building intuitive UIs and managing responsive user interactions.

## Getting Started

### Prerequisites

Before using Pincer Chat, you'll need to install Ollama to serve the LLM:

1. Download and install [Ollama](https://ollama.com/download).
2. Start the Ollama server by running:

   ```shell
   ollama serve
   ```

### Installing Pincer Chat

To get started with Pincer Chat, install the application using Cargo:

1. Install the app:

   ```shell
   cargo install --git https://github.com/AnesBenmerzoug/pincer-chat.git
   ```

2. Start the application by running:

   ```shell
   pincer-chat
   ```

This will launch the desktop GUI where you can begin interacting with the local LLM.

### Troubleshooting

If you encounter issues, ensure that:

- Ollama is properly installed and running.
- You have the latest version of Rust and Cargo.

# License 

This project is licensed under the [LGPL-2.1](https://www.gnu.org/licenses/old-licenses/lgpl-2.1.en.html) license.