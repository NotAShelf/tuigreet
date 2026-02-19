{tuigreet-pkg}: {
  baseMachine = {
    users.users.alice = {
      isNormalUser = true;
      description = "Alice Test";
      password = "test123";
      uid = 1000;
    };

    users.users.bob = {
      isNormalUser = true;
      description = "Bob Test";
      password = "test456";
      uid = 1001;
    };

    # Create cache directory for tuigreet remember features
    # FIXME: use the new tmpfiles API when I can be arsed to
    # look it up.
    systemd.tmpfiles.rules = [
      "d /var/cache/tuigreet 0755 greeter greeter -"
    ];

    environment = {
      systemPackages = [tuigreet-pkg];

      # Create minimal session files for testing
      etc."wayland-sessions/sway.desktop".text = ''
        [Desktop Entry]
        Name=Sway
        Comment=An i3-compatible Wayland compositor
        Exec=sway
        Type=Application
      '';

      etc."xsessions/xfce.desktop".text = ''
        [Desktop Entry]
        Name=Xfce Session
        Exec=startxfce4
        Type=Application
      '';
    };
  };
}
