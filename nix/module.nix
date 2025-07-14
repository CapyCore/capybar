self: {
    lib,
    pkgs,
    config,
    ...
}: let
    cfg = config.programs.capybar;
in with lib; {
    options.programs.capybar = {
        enable = mkEnableOption "capybar";

        package = mkOption {
            type = types.package;
            description = "The capybar package to use.";
            default = self.packages.${pkgs.system}.capybar;
        };
    };

    config = mkIf cfg.enable {
        home.packages = [ cfg.package ];

#        xdg.configFile."capybar/config.toml" = {
#            text = builtins.toTOML cfg.settings;
#        };
    };
}
