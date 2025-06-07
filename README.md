# Capybar
> [!CAUTION]
> Bar is in very early stages of development. Everything might change and nothing might work for you
>
> Right now only hyprland was tested. Other compositors could work wrong, althogh most likely won't

Simple customizable bar applications that aims to have as little external dependencies (like gtk, qt, upowerd etc.) as possible. 

## Features
- Custom widgets creation via rust
- Pre-built widgets:
    - Text
    - Clock
    - Battery
    - CPU usage
    - Row container (WIP)
    - Bar container

## Instalation
Currently bar needs to be build manually. To do so clone the repo and write main file. Bulding the bar is done with cargo. The example is located in examples folder. To use the basic example run:
```
cargo build --release --example basic
```

## Usage
After building the bar the executable will be located in `./target/release/`

The basic example exetutable is `./target/release/examples/basic`


## License

Capybar is licensed under the MIT license. [See LICENSE for more information](https://github.com/YggdraCraft/capybar/blob/master/LICENSE).


## Acknowledgements

- [Smithay Client Toolkit](https://github.com/Smithay/client-toolkit) - Base for all the wayland communication and the bar itself
