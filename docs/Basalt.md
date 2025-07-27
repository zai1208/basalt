Basalt is a TUI (Terminal User Interface) application to manage Obsidian vaults and notes from the terminal. Basalt is cross-platform and can be installed and run in the major operating systems on Windows, macOS and Linux.

Basalt is not a complete or comprehensive replacement for Obsidian, but instead a minimalist approach for note management in terminal with a readable markdown rendering and [WYSIWYG](https://en.wikipedia.org/wiki/WYSIWYG) experience.

## Vision

- Basalt functions as a companion app for Obsidian that enables quick note editing without interrupting the terminal flow
- Basalt enables text editing in a familiar way (Obsidian, vim) without having to rely on external editors
- Basalt is a terminal based [WYSIWYG](https://en.wikipedia.org/wiki/WYSIWYG) markdown editor
- Basalt works as a CLI for finding / deleting / creating notes and works with the rest of the unix tooling
- Basalt is a standalone terminal note managing application that works seamlessly with Obsidian

## Background

This is something that has been brewing in my head for quite some time. There has been different incarnations over the years, however, nothing as substantial as this.

I have been using Neovim and the official Obsidian app. However, I wanted to have something dedicated that offers the same writing experience as Neovim, but has more WYSIWYG experience as in the official Obsidian app. I'm fully aware of [obsidian.nvim](https://github.com/epwalsh/obsidian.nvim), which many people use and find more than sufficient. However, I want to see images, beautified text, note graphs, etc. I want it to be a bit more.

The problem for me personally is that when I leave the terminal, my flow breaks, especially if I'm writing. Using an entirely different app disrupts that flow, and it _annoys_ me. So here I am, building a TUI for Obsidian.

The goal of basalt is not to replace the Obsidian app. Basalt is to fill and cater a need to have a terminal view to the selection of notes and vaults, providing quick access from anywhere in the terminal with a simple command.

## Architecture

> [!CAUTION]
>
> TBD (Elm)
