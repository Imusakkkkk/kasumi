 <p align="center">
  <img width="32%" src="PATH_TO_KASUMI_LOGO" alt="kasumi logo"/>
</p>

<p align="center">
  <a href="https://github.com/Imusakkkkk/Kasumi/blob/main/license.txt">
    <img alt="License" src="https://img.shields.io/badge/license-GPL--3.0-blue.svg?style=flat">
  </a>
  <a href="https://github.com/Imusakkkkk/Kasumi">
    <img alt="GitHub" src="https://img.shields.io/badge/project-kasumi-8A8A8A?style=flat&logo=github">
  </a>
</p>

<h1 align="center">kasumi</h1>
<h4 align="center">a very experimental nintendo switch 2 emulator project</h4>
<h5 align="center">written lowercase.</h5>

kasumi is an extremely experimental nintendo switch 2 emulator written primarily in rust, with a Qt 6 debugging frontend.

based on the original [oboromi](https://github.com/0xNikilite/oboromi) project, kasumi aims to independently continue exploring and developing the foundations of nintendo switch 2 emulation.

## current focus

we are developing a new [ARMv9 CPU emulator](https://github.com/vrtgs/rapid-arm-emu) (original work by vrtgs), targeting the ARM extensions required by the nintendo switch 2's T239.

the GPU implementation targets NVIDIA SM86.

kasumi is still in a very early experimental stage. it cannot currently run commercial games or firmware.

## repository

- `core/` — rust emulator core
- `gui-qt/` — c++ Qt 6 frontend
- `scripts/` — code-generation and helper scripts
- `docs/` — documentation
- `benchmarks/` — performance benchmarks
- `examples/` — examples and experiments
- `assets/` — project assets

## building

requirements: stable rust, cmake, ninja, a c++ compiler, and Qt 6.

see [`CONTRIBUTING.md`](CONTRIBUTING.md) for setup and build instructions.

## project origins

kasumi is an independent project based on the [oboromi](https://github.com/0xNikilite/oboromi) codebase.

the original project and its contributors deserve credit for the work that made kasumi possible.

## license

GPL-3.0. see [`license.txt`](license.txt).

## what does the name mean?

**kasumi (霞)** is a japanese word meaning **"mist"** or **"haze."**

the name represents something that is not yet fully clear: an experimental project exploring the unknown, gradually taking shape through research, development, and experimentation.

like the mist, kasumi is still evolving. its final form is not yet defined.
