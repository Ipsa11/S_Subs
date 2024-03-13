# The SaitaChain Parachain Host Implementers' Guide

The implementers' guide is compiled from several source files with [`mdBook`](https://github.com/rust-lang/mdBook).

## Hosted build

## Local build

To view it locally from the repo root:

Ensure graphviz is installed:

```sh
brew install graphviz # for macOS
sudo apt-get install graphviz # for Ubuntu/Debian
```

Then install and build the book:

```sh
cargo install mdbook mdbook-linkcheck mdbook-graphviz mdbook-mermaid mdbook-last-changed
mdbook serve roadmap/implementers-guide
```

and in a second terminal window run:

```sh
open http://localhost:3000
```

