# :crab: Rustacean Editor :crab:

## What is it?
A very basic editor written in Rust with the awesome egui library to explore immediate mode guis. This project was made with the awesome egui template which can be found [here](https://github.com/emilk/eframe_template).

## :warning: Important information :warning:
Some stuff does not work properly yet:
* File related stuff does not work on WASM target
* Changing the font size only affects the text inside the editor window and not the global font size

## Prerequisites
On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel libxkbcommon-devel pkg-config openssl-devel libxcb-devel`

For running the `build_web.sh` script you also need to install `jq` and `binaryen` with your packet manager of choice.

For example binaryen can be installed with npm by running `npm i -g binaryen`

The `jq` command can be found [here](https://stedolan.github.io/jq/download/).

## Creating a release build

Make sure you are using the latest version of stable rust by running `rustup update`.

`cargo run --release`

## Compiling for the web

Make sure you are using the latest version of stable rust by running `rustup update`.

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page. For this you need to set up some tools (see prerequisites). There are a few simple scripts that help you with this:

```sh
./setup_web.sh
./build_web.sh
./start_server.sh
open http://127.0.0.1:8080/
```

* `setup_web.sh` installs the tools required to build for web
* `build_web.sh` compiles your code to wasm and puts it in the `docs/` folder (see below)
* `start_server.sh` starts a local HTTP server so you can test before you publish
* Open http://127.0.0.1:8080/ in a web browser to view

Generally there are three important files:
* `index.html`: A few lines of HTML, CSS and JS that loads the app.
* `rustacean_editor.wasm`: What the Rust code compiles to.
* `rustacean_editor.js`: Auto-generated binding between Rust and JS.