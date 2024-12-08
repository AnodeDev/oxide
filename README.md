# OXIDE

⚠️ VERY EARLY IN DEVELOPMENT! ⚠️

Don't have any high expectations, right now I'm just building it for myself

## About

This is my own terminal text editor that I built to work around the annoying rebinding experience of Neovim and Emacs.

## The Problem

I use the Colemak-DH layout, so some of the regular keybindings for Neovim are really not good, for example `hjkl`, and the same goes for a lot of keybindings in Emacs as well. I use a Corne keyboard, and I use the Alt key instead of Super in sxhkd for my keybindings, which makes some Emacs bindings impossible to do. On top of that, changing the default keybindings for Neovim is a real pain, since some bindings are so deeply nested in a certain mode, like `i` in visual mode.

This made it really annoying to work in Neovim and Emacs, since I had to switch between using `neio` and the arrow keys for moving the cursor, and it made configuring a nightmare.

## The Solution

I'll just make my own editor... It's that easy!

I've started working on a keybinding design that hopefully makes this a lot less of a hassle, meaning you can easily configure the keybindings to fit any layout.

Since I already knew I wanted to build this, I decided to add some extra functions as well. I really wanted to make the editor as fast as possible, and since I'm building it in Rust I want to try to implement `async` as I don't want to have any actions that might freeze the editor. 

I really like the package system in Emacs, so I want to try to make my own version that can install and load packages asynchronously, hopefully meaning startup time for the editor is a lot shorter. With this I really wanted a cool way to see the packages and their dependencies, so I thought I'd try to make a tree structure for the packages where the user can see all packages, dependencies, conflicts and interdependencies to help manage the packages effectively.

I want to implement a really memory-efficient buffer system, and I settled on the idea of storing the buffers as deltas instead of the whole file. This means the only data that is stored is the changes, making the size of the buffer really small. I also wanted to try to implement a compression system, compressing large files or buffers that have been inactive for a long time to free up memory.

For UI I want to use [ratatui](https://github.com/ratatui/ratatui), since it makes it really easy to make cool UI systems for floating windows and such.

## Current Status

I'm currently working on the config language for the editor, [patina](https://github.com/AnodeDev/patina). After that's done I can clean up some of the code and add some documentation on how configuration works.

## Roadmap

- [x] Make the basic editor with buffer system :)
- [x] Implement the modal system + keybinding system
- [ ] Delta buffer storage + async write-back
- [ ] Async package manager
- [ ] Async buffer compression and decompression
- [ ] Refine UI with async event handling
- [ ] Done :)
