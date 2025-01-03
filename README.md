# OXIDE

⚠️ VERY EARLY IN DEVELOPMENT! ⚠️

Don't have any high expectations, right now I'm just building it for myself

---

## About

This is my own terminal text editor that I built to work around the annoying rebinding experience of Neovim and Emacs.

---

## The Problem

I use the Colemak-DH layout, so some of the regular keybindings for Neovim are really not good, for example `hjkl`, and the same goes for a lot of keybindings in Emacs as well. I use a Corne keyboard, and I use the Alt key instead of Super in sxhkd for my keybindings, which makes some Emacs bindings impossible to do. On top of that, changing the default keybindings for Neovim is a real pain, since some bindings are so deeply nested in a certain mode, like `i` in visual mode.

This made it really annoying to work in Neovim and Emacs, since I had to switch between using `neio` and the arrow keys for moving the cursor, and it made configuring a nightmare.

## The Solution

I'll just make my own editor... It's that easy!

The keybinding system I have developed is much less of a hassle to configure, at least once the editor can be configured properly. I also want to make some sort of a visualizer for which keys are bound where in the future.

The editor will be written in Rust (not because I'm a Rust-bro but because I like writing in it), so I want to it to it's fullest capacity and implement some QoL features to make the editor feel as smooth as possible. I plan on achieving this through the use of `async` for actions that might take a while, like loading and writing to files.

I really like the package system in Neovim, so I want to try to make my own version that can install and load packages asynchronously, hopefully meaning startup time for the editor is a lot shorter. With this I really want a cool way to see the packages and their dependencies, so I'm planning on making a tree structure for the packages where the user can see all packages, dependencies, conflicts and interdependencies to help manage the packages effectively.

I want to implement a really memory-efficient buffer system, and I settled on the idea of storing the buffers as deltas instead of the whole file. This means the only data that is stored is the changes, making the size of the buffer really small. I also want to try to implement a compression system, compressing large files or buffers that have been inactive for a long time to free up memory.

For UI I want to use [ratatui](https://github.com/ratatui/ratatui), since it makes it really easy to make cool UI systems for floating windows and such.

---

## Project Status
**current_state:** Active Development

Oxide is currently in very early beta.

- Implemented Features:
    - Basic vi-like modal editing
    - Basic file editing (loading file, saving, etc.)
    - Keybindings
    - Emacs-like minibuffer
    - Buffer switching

- In Progress:
    - Rewrite rendering engine
    - Config language ([patina](https://github.com/AnodeDev/patina))
    - Plugin system

- Upcomming:
    - Buffer compression system

## Contributing

Contributions are very much appreciated! This is quite a large project and I'm just one person, so any help is much appreciated.

If you want to contribute, make sure to read the [contributing](https://github.com/AnodeDev/oxide/blob/main/CONTRIBUTING.md) guidelines before creating a pull request.
