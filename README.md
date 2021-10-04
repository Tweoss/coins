# ToC
<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->

<!-- code_chunk_output -->

- [ToC](#toc)
- [Overall Description](#overall-description)
- [Server](#server)
  - [Files](#files)
  - [Running](#running)
- [Login Page](#login-page)
  - [Files](#files-1)
  - [Compiling](#compiling)
- [Game](#game)
  - [Files](#files-2)
  - [Compiling](#compiling-1)
- [View](#view)
  - [Files](#files-3)
  - [Compiling](#compiling-2)
- [CLI](#cli)
  - [Files](#files-4)
  - [Running](#running-1)
- [Text CLI](#text-cli)
  - [Files](#files-5)
  - [Running](#running-2)

<!-- /code_chunk_output -->
 
# Overall Description
Server and client code to have a multiplayer simulation of coin flipping. Demonstrates three algorithms: Thompson sampling, Upper Confidence Bound, and Naive Bayes. Problem is also called Multi-Armed Bandits. Notable crates: [serde](https://serde.rs/), [actix](https://actix.rs), and [mogwai](https://github.com/schell/mogwai).

# Server

Basic [Actix](https://actix.rs/) server that manages coin flipping for clients. Takes three proportions corresponding to the probablity of head for each of three coins. Each time a coin is flipped by a client (via an HTTP GET request), the server sends a message to an [Actix actor](https://actix.rs/actix/actix/trait.Actor.html) to run all three algorithms (Thompson sampling, Naive Bayes, and Upper Confidence Bound) and update their individual states. A POST request to `/flush` causes the actor to dump its state, including the algorithm choices and the player choices, into `dump.cbor`.

## Files
| File                                    | Description                                                                                   |
|-----------------------------------------|-----------------------------------------------------------------------------------------------|
| [app.rs](./server/src/app.rs)           | Logic for the actor. Includes messages, message handling, and algorithm update logic.         |
| [cli.rs](./server/src/cli.rs)           | Basic [Clap](https://docs.rs/clap/2.33.3/clap/) struct for parsing the command line arguments |
| [handlers.rs](./server/src/handlers.rs) | Handlers for HTTP requests (static files, flushing, sending messages to actor)                |
| [main.rs](./server/src/main.rs)         | What do you think? It's main.                                                                 |
| [dump.cbor](./server/dump.cbor)         | The output after flushing. [Binary JSON file](https://docs.rs/serde_cbor/0.11.2/serde_cbor/). |

## Running
```bash
cd server; 
cargo run 0.3 0.2 0.9;
# open 0.0.0.0:8080
```

# Login Page

Login page using Mogwai's frontend framework. Styled and designed with Bootstrap Studio. Submits username for a cookie to be set then redirects to the Game page.

## Files
| File                                   | Description                                                                                                 |
|----------------------------------------|-------------------------------------------------------------------------------------------------------------|
| [lib.rs](./login/src/lib.rs)           | The main file. Builds a component and runs it, with channels for changing username and submitting username. |
| [styles.css](./login/style/styles.css) | Custom styling                                                                                              |
| [index.html](./login/index.html)       | File that imports and runs the javascript shim for WebAssembly                                              |

## Compiling
```bash
cd login;
wasm-pack build --target=web;
```

# Game

Game page also using Mogwai. 

## Files
| File                                  | Description                                                 |
|---------------------------------------|-------------------------------------------------------------|
| [lib.rs](./game/src/lib.rs)           | Just like login page, main file.                            |
| [coin.rs](./game/src/coin.rs)         | Declares a coin component and its messages. Used in lib.rs. |
| [styles.css](./game/style/styles.css) | Custom styling again                                        |
| [index.html](./game/index.html)       | HTML document, imports javascript and runs the WebAssembly. |

## Compiling
```bash
cd game;
wasm-pack build --target=web;
```

# View

A page also running with Mogwai that accepts a `.cbor` file (generated from the Text CLI) and graphs it in an SVG. Demonstrates the evolution of the algorithms and shows the time-varying preferences of the top player and algorithms. 

## Files
| File                                  | Description                                        |
|---------------------------------------|----------------------------------------------------|
| [lib.rs](./view/src/lib.rs)           | Just like login page and game page, main file.     |
| [styles.css](./view/style/styles.css) | Custom styling again                               |
| [index.html](./view/index.html)       | HTML file runs the WebAssembly compiled from Rust. |

## Compiling
```bash
cd view;
wasm-pack build --target=web;
```

# CLI

CLI to generate images from `dump.cbor`. Generates as many images as there are choices in the images directory. 

## Files
| File                                  | Description                                        |
|---------------------------------------|----------------------------------------------------|
| [main.rs](./cli/src/main.rs)           | Main file, runs functions from utils.rs    |
| [utils.rs](./cli/src/utils.rs)           | Utils.rs, determines how algorithms would sample  |
| [images](./cli/images)           | Output directory |

## Running
```bash
cd cli;
cargo run;
```

# Text CLI

CLI to generate a `rendered_dump.cbor` to be loaded and viewed via the view page. Limits the number of iterations to 100 or the number of iterations given. The resolution of the output path is, by default, 80 points per beta distribution. The output is placed in `rendered_dump.cbor`. 

## Files
| File | Description | 
| -- | --| 
| [main.rs](./text_cli/src/main.rs) | Main file, runs functions from utils.rs. Reads the server dump and generates its own. Takes the maximum iterations on cli. |
| [utils.rs](./text_cli/src/utils.rs) | Contains the default 80 points per beta distribution as a `const`. Determines how the algorithm would view the state. |

## Running 

```bash
cd text_cli;
cargo run 500;
```