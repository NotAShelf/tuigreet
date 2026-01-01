# Test: Basic functionality - tuigreet starts and renders UI
{
  lib,
  testers,
  tuigreet-pkg,
}: let
  common = import ./common.nix {inherit tuigreet-pkg;};
in
  testers.nixosTest {
    name = "tuigreet-basic";
    meta.maintainers = with lib.maintainers; [NotAShelf];

    nodes.machine = lib.mkMerge [
      common.baseMachine
      {
        services.greetd = {
          enable = true;
          settings = {
            terminal.vt = 1;
            default_session = {
              command = "${tuigreet-pkg}/bin/tuigreet --greeting 'Test Greeting' --time --cmd sway";
              user = "greeter";
            };
          };
        };
      }
    ];

    testScript = ''
      machine.wait_for_unit("greetd.service")
      machine.wait_for_unit("getty@tty1.service")

      # Check that tuigreet is running. This means greetd has successfully
      # handed off to tuigreet, and the process hasn't crashed or anything.
      machine.succeed("pgrep -f tuigreet")

      # Try to capture tty1 output and verify tuigreet UI is rendered.
      # We should see the greeting text
      machine.wait_until_succeeds("ps aux | grep tuigreet | grep -v grep")

      # Check that the screen shows our custom greeting
      # NOTE: Afaik we cannot easily capture tty1 in NixOS tests, so we verify that
      # the process is running and has the correct arguments. This is, in fact, a
      # basic test as the name indicates.
      output = machine.succeed("ps aux | grep tuigreet | grep -v grep")
      assert "Test Greeting" in output, "tuigreet should have custom greeting"
      assert "--time" in output, "tuigreet should show time"
      assert "--cmd sway" in output, "tuigreet should have default command"
    '';
  }
