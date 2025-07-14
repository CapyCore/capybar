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
    - Keyboard layout
    - Row container (WIP)
    - Bar container

## Instalation

### Nix

Capybar can be installed on nix using home manager.
- Extend your inputs with:
```nix
 inputs = {
    # ...
    capybar.url = "github:CapyCore/capybar"; 
  };
```

- Extend your imports with:
```nix
imports = [ inputs.capybar.homeManagerModules.default ];
```

- Enable capybar:
```nix
programs.capybar = {
    enable = true;
}
```

### Others
Currently bar needs to be build manually. To do so clone the repo and write main file. Bulding the bar is done with cargo. The example is located in examples folder.
```
cargo build --release
```

After building the bar the executable will be located in `./target/release/`

## Usage

Capybar can be run using `capybar` command in a terminal of your choice. You can change configuration path via flag 
`--cfg_path` (default path is `$HOME/.config/capybar`) and config extention via `--cfg_type` (default is toml, no other types are
currently supported). More info could be accesed wit `--help` flag.

## License

Capybar is licensed under the MIT license. [See LICENSE for more information](https://github.com/YggdraCraft/capybar/blob/master/LICENSE).


## Acknowledgements

- [Smithay Client Toolkit](https://github.com/Smithay/client-toolkit) - Base for all the wayland communication and the bar itself
