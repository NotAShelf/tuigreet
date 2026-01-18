# tuigreet

[greetd]: https://git.sr.ht/~kennylevinsen/greetd
[tuigreet]: https://github.com/apognu/tuigreet
[features]: #features

Graphical console greeter for [greetd], fork of [tuigreet] for a more modern and
hackable codebase suitable for future extension.

> [!IMPORTANT]
> This repository has been forked from [tuigreet] due to the upstream
> inactivity. While I _do_ hope that upstream comes back alive eventually, I
> have elected to maintain this fork for the foreseeable future. My main
> motivation is that tuigreet is not very good at launching graphical sessions;
> if I want to handle `graphical-session.target` and friends, I'm required to
> use a session wrapper like UWSM or write my own wrapper script. I find the
> status quo to be less than desirable, as I'm already using a login manager and
> a greeter. Why should I wrap for the 3rd time? With this in mind, this
> repository has been created as a fork to maintain tuigreet on my own time
> while incrementally improving the codebase, merging old PRs that have been
> stale for too long, fixing bugs and adding more features as I need them. _If_
> you are interested in using this, great. Let me know what you need, and I'll
> see what I can do for you. If you want to contribute, that's even better! Open
> a PR, and let's see where it takes us.
>
> The [features] section below shall contain a semi-maintained list of new
> features on top of the old features that I wanted to document explicitly.

## Features

tuigreet provides a terminal-based authentication interface with session
management, user selection, and power controls. The upstream project includes
session launching from desktop files, username/session persistence, NSS-backed
user menus, themeable UI components, and multi-language support.

This fork, as per its motivation to maintain tuigreet with much-desired features
and stability, extends the original with TOML-based configuration (supporting
both user and system config files with hot-reload), environment variable mapping
for all options, detailed error messages with source context for config issues,
and exposes core functionality as a library.

Additional, and perhaps less relevant, work includes NixOS VM-based integration
tests, various bug fixes (session wrapper behavior, UID handling, padding
semantics, status bar rendering) and so on.

## Usage

![Screenshot of tuigreet](https://github.com/notashelf/tuigreet/blob/master/contrib/assets/screenshot.png)

The default configuration of tuigreet is quite minimal, visually speaking. It
only displays the authentication prompt and some minor information in the status
bar. You may additionally print your system's `/etc/issue` at the top of the
prompt with `--issue`, and the current date & time using `--time`. The time can
also be customized with the `--time-format` flag. It is also possible to include
a custom, one-line greeting message _instead of_ `/etc/issue` using the
`--greeting` flag.

### Prompt Customization

The initial prompt container will be 80 columns wide. You might want to change
this using the `--width` flag in the case you need more space, e.g., to account
for larger PAM challenge messages. Please refer to usage information (`--help`)
for more customization options. Various padding settings are available through
the `*-padding` options.

### Session Persistence

You can instruct `tuigreet` to remember the last username that successfully
opened a session with the `--remember` option (that way, the username field will
be pre-filled). Similarly, the command and session configuration can be retained
between runs with the `--remember-session` option (when using this, the `--cmd`
value is overridden by manual selections). You can also remember the selected
session per user with the `--remember-user-session` flag. In this case, the
selected session will only be saved on successful authentication.

You may change the command that will be executed after opening a session by
hitting `F2` and amending the command. Alternatively, you can list the
system-declared sessions (or custom ones) by hitting `F3`. Power options are
available through `F12`.

## Install

This fork is currently not packaged anywhere. A Nix flake is provided, and you
may build from source if you are interested in using the fork. Should you wish
to package this for your distribution, do feel free to update the readme with
per-distribution instructions.

### With Nix

This fork is not packaged in Nixpkgs, but it is trivial to use the Nixpkgs
derivation with the updated source information, should you wish to run it. For
example, you may create an overlay to override `pkgs.tuigreet` as follows:

```nix
[
   (prev: {
      tuigreet = prev.tuigreet.overrideAttrs {
         src = prev.fetchFromGitHub {
            owner = "NotAShelf";
            repo = "tuigreet";
            tag = "0.9.2"; # update this with the tag you want to use
            hash = ""; # update this with the appropriate hash for your tag
         };
      };
   }
]
```

Additionally you may get it from the flake provided in this repository. In which
case you may use the `default` or `tuigreet` packages. The easiest way of doing
so is creating an overlay as above but point `tuigreet` to
`inputs.tuigreet.packages.${prev.hostPlatform.system}.tuigreet` instead of
overriding the `src`. This will completely replace the derivation, and build
with the correct source automatically.

### From source

Building Tuigreet from source requires an installation of Rust's stable
toolchain. Currently 1.90 and above is required. You may use the Nix devshell
provided by the repository, or install it using something like `rustup`.

```sh
# Clone the repository and navigate to it
$ git clone https://github.com/NotAShelf/tuigreet && cd tuigreet

# Build in release mode
$ cargo build --release

# You may then move it to somewhere you can use it. If on NixOS, refer to above
# steps instead of trying to copy the binary.
# $ mv target/release/tuigreet /usr/local/bin/tuigreet
```

> [!NOTE]
> Cache directory must be created for `--remember*` features to work. The
> directory must be owned by the user running the greeter.

```bash
# If cache is missing or owned by the wrong user, you may run the following
# commands to create it, or to fix the permissions.
$ mkdir /var/cache/tuigreet
$ chown greeter:greeter /var/cache/tuigreet
$ chmod 0755 /var/cache/tuigreet
```

### Pre-built binaries

Pre-built binaries of `tuigreet` for several architectures can be found in the
[releases](https://github.com/NotAShelf/tuigreet/releases) section of this
repository. The
[tip prerelease](https://github.com/NotAShelf/tuigreet/releases/tag/tip) is
continuously built and kept in sync with the `master` branch.

## Running the tests

Tests from the default features should run without any special consideration by
running `cargo test`.

If you intend to run the whole test suite, you will need to perform some setup.
One of our features uses NSS to list and filter existing users on the system,
and in order not to rely on actual users being created on the host, we use
[libnss_wrapper](https://cwrap.org/nss_wrapper.html) to mock responses from NSS.
Without this, the tests would use the real user list from your system and
probably fail because it cannot find the one it looks for.

```bash
# After installing `libnss_wrapper` on your system (or compiling it to get the`.so`)
# you can run those specific tests as such:
$ export NSS_WRAPPER_PASSWD=contrib/fixtures/passwd
$ export NSS_WRAPPER_GROUP=contrib/fixtures/group
$ LD_PRELOAD=/path/to/libnss_wrapper.so cargo test --features nsswrapper nsswrapper_ # to run those tests specifically
$ LD_PRELOAD=/path/to/libnss_wrapper.so cargo test --all-features # to run the whole test suite
```

## Configuration

Edit `/etc/greetd/config.toml` and set the `command` setting to use `tuigreet`:

```toml
[terminal]
vt = 1

[default_session]
command = "tuigreet --cmd sway"
user = "greeter"
```

Please refer to [greetd's wiki](https://man.sr.ht/~kennylevinsen/greetd/) for
more information on setting up `greetd`.

### TOML Configuration

`tuigreet` supports TOML configuration files in addition to command-line
options. Configuration files are loaded from:

1. `~/.config/tuigreet/config.toml` (user config)
2. `/etc/tuigreet/config.toml` (system config)
3. Custom path via `--config <path>`

Configuration priority: CLI args > environment variables > user config > system
config > defaults

#### Configuration Example

```toml
[display]
show_time = true
greeting = "Welcome to the system!"
align_greeting = "center"
issue = false

[layout]
width = 60
window_padding = 2
container_padding = 1
prompt_padding = 1

[layout.widgets]
time_position = "top"      # "top", "bottom", "default", "hidden"
status_position = "bottom" # "top", "bottom", "default", "hidden"

[remember]
username = true
session = false
user_session = true

[user_menu]
enabled = true
min_uid = 1000
max_uid = 60000

[secret]
mode = "characters"  # "hidden" or "characters"
characters = "*"

[keybindings]
command = 2   # F2
sessions = 3  # F3
power = 12    # F12

[session]
sessions_dirs = ["/usr/share/wayland-sessions", "/usr/share/xsessions"]
xsessions_dirs = []
environments = []

[power]
use_setsid = false

[theme]
border = "white"
text = "green"
time = "blue"
container = "black"
title = "cyan"
greet = "yellow"
prompt = "magenta"
input = "white"
action = "bright-blue"
button = "bright-red"
```

#### Environment Variables

All configuration options can also be set via environment variables. The naming
convention is `TUIGREET_<SECTION>_<KEY>` for nested options, or `TUIGREET_<KEY>`
for top-level options:

```bash
# General configuration
export TUIGREET_DEBUG=true
export TUIGREET_LOG_FILE="/custom/path/tuigreet.log"

# Display options
export TUIGREET_TIME=true
export TUIGREET_TIME_FORMAT="%Y-%m-%d %H:%M"
export TUIGREET_GREETING="Welcome!"
export TUIGREET_ISSUE=false
export TUIGREET_ALIGN_GREETING=center  # left, center, right

# Layout configuration
export TUIGREET_WIDTH=80
export TUIGREET_WINDOW_PADDING=1
export TUIGREET_CONTAINER_PADDING=1
export TUIGREET_PROMPT_PADDING=1

# Widget positioning
export TUIGREET_TIME_POSITION=top      # default, top, bottom, hidden
export TUIGREET_STATUS_POSITION=bottom # default, top, bottom, hidden

# Remember options
export TUIGREET_REMEMBER_USERNAME=true
export TUIGREET_REMEMBER_SESSION=false
export TUIGREET_REMEMBER_USER_SESSION=true

# User menu configuration
export TUIGREET_USER_MENU=true
export TUIGREET_MIN_UID=1000
export TUIGREET_MAX_UID=60000

# Secret display
export TUIGREET_SECRET_MODE=characters  # hidden, characters
export TUIGREET_SECRET_CHARACTERS="●"

# Session configuration
export TUIGREET_COMMAND="sway"
export TUIGREET_SESSIONS_DIRS="/usr/share/wayland-sessions:/custom/sessions"
export TUIGREET_XSESSIONS_DIRS="/usr/share/xsessions"
export TUIGREET_SESSION_WRAPPER="systemd-cat -t sway"
export TUIGREET_XSESSION_WRAPPER="startx"
export TUIGREET_ENVIRONMENTS="WAYLAND_DISPLAY:DISPLAY"

# Power options
export TUIGREET_USE_SETSID=false

# Keybindings (F-key numbers)
export TUIGREET_KB_COMMAND=2   # F2
export TUIGREET_KB_SESSIONS=3  # F3
export TUIGREET_KB_POWER=12    # F12

# Individual theme components
export TUIGREET_THEME_BORDER=white
export TUIGREET_THEME_TEXT=green
export TUIGREET_THEME_TIME=blue
export TUIGREET_THEME_CONTAINER=black
export TUIGREET_THEME_TITLE=cyan
export TUIGREET_THEME_GREET=yellow
export TUIGREET_THEME_PROMPT=magenta
export TUIGREET_THEME_INPUT=white
export TUIGREET_THEME_ACTION=bright-blue
export TUIGREET_THEME_BUTTON=bright-red

# Or use legacy theme format (semicolon-separated)
export TUIGREET_THEME="border=white;text=green;time=blue;container=black"
```

#### Hot Reload

Configuration files are automatically monitored for changes and hot-reloaded
when modified. This allows you to adjust settings without restarting the
greeter.

#### Configuration Errors

tuigreet makes an effort to include detailed context with line numbers and
source code snippets to help identify and fix configuration issues. For example:

```plaintext
TOML parsing error at line 5, column 15:
   4 | [layout]
   5 | width = "invalid_number"
        |         ^^^^^^^^^^^^^^^^ expected integer, found string
   6 | window_padding = 2
```

### Sessions

The available sessions are fetched from `desktop` files in
`/usr/share/xsessions` and `/usr/share/wayland-sessions`. If you want to provide
custom directories, you can set the `--sessions` arguments with a
colon-separated list of directories for `tuigreet` to fetch session definitions
some other place.

#### Desktop environments

`greetd` only accepts environment-less commands to be used to start a session.
Therefore, if your desktop environment requires either arguments or environment
variables, you will need to create a wrapper script and refer to it in an
appropriate desktop file.

For example, to run X11 Gnome, you may need to start it through `startx` and
configure your `~/.xinitrc` (or an external `xinitrc` with a wrapper script):

```plaintext
exec gnome-session
```

To run Wayland Gnome, you would need to create a wrapper script akin to the
following:

```bash
XDG_SESSION_TYPE=wayland dbus-run-session gnome-session
```

Then refer to your wrapper script in a custom desktop file (in a directory
declared with the `-s/--sessions` option):

```plaintext
Name=Wayland Gnome
Exec=/path/to/my/wrapper.sh
```

#### Common wrappers

Two options allows you to automatically wrap run commands around sessions
started from desktop files, depending on whether they come
`/usr/share/wayland-sessions` or `/usr/share/xsessions`: `--sessions-wrapper`
and `--xsessions-wrapper`. With this, you can prepend another command on front
of the sessions you run to set up the required environment to run these kinds of
sessions.

By default, unless you change it, all X11 sessions (those picked up from
`/usr/share/xsessions`) are prepended with `startx /usr/bin/env`, so the X11
server is started properly.

### Power management

Two power actions are possible from `tuigreet`, shutting down (through
`shutdown -h now`) and rebooting (with `shutdown -r now`) the machine. This
requires that those commands be executable by regular users, which is not the
case on some distros.

To alleviate this, there are two options that can be used to customize the
commands that are run: `--power-shutdown` and `--power-reboot`. The provided
commands must be non-interactive, meaning they will not be able to print
anything or prompt for anything. If you need to use `sudo` or `doas`, they will
need to be configured to run passwordless for those specific commands.

An example for `/etc/greetd/config.toml`:

```toml
[default_session]
command = "tuigreet --power-shutdown 'sudo systemctl poweroff'"
```

> [!NOTE]
> By default, all commands are prefixed with `setsid` to completely detach the
> command from our TTY. If you would prefer to run the commands as is, or if
> `setsid` does not exist on your system, you can use `--power-no-setsid`.

### User menu

Optionally, a user can be selected from a menu instead of typing out their name,
with the `--user-menu` option, this will present all users returned by NSS at
the time `tuigreet` was run, with a UID within the acceptable range. The values
for the minimum and maximum UIDs are selected as follows, for each value:

- A user-provided value, through `--user-menu-min-uid` or `--user-menu-max-uid`;
- **Or**, the available values for `UID_MIN` or `UID_MAX` from
  `/etc/login.defs`;
- **Or**, hardcoded `1000` for minimum UID and `60000` for maximum UID.

### Theming

[in the ratatui repository]: https://github.com/ratatui/ratatui/blob/main/ratatui-core/src/style/color.rs

A theme specification can be given through the `--theme` argument to control
some of the colors used to draw the UI. This specification string must have the
following format: `component1=color;component2=color[;...]` where the component
is one of the value listed in the table below, and the color is a valid ANSI
color name as listed [in the ratatui repository].

Mind that the specification string include semicolons, which are command
delimiters in most shells, hence, you should enclose it in single-quotes so it
is considered a single argument instead.

Please note that we can only render colors as supported by the running terminal.
In the case of the Linux virtual console, those colors might not look as good as
one may think. Your mileage may vary.

<!-- markdownlint-disable MD013 -->

| Component name | Description                                                                        |
| -------------- | ---------------------------------------------------------------------------------- |
| text           | Base text color other than those specified below                                   |
| time           | Color of the date and time. If unspecified, falls back to `text`                   |
| container      | Background color for the centered containers used throughout the app               |
| border         | Color of the borders of those containers                                           |
| title          | Color of the containers' titles. If unspecified, falls back to `border`            |
| greet          | Color of the issue of greeting message. If unspecified, falls back to `text`       |
| prompt         | Color of the prompt ("Username:", etc.)                                            |
| input          | Color of user input feedback                                                       |
| action         | Color of the actions displayed at the bottom of the screen                         |
| button         | Color of the keybindings for those actions. If unspecified, falls back to `action` |

<!-- markdownlint-enable MD013 -->

Below is a screenshot of the greeter with the following theme applied:

```plaintext
`border=magenta;text=cyan;prompt=green;time=red;action=blue;button=yellow;container=black;input=red`:
```

Which results in the following:

![Screenshot of tuigreet](https://github.com/NotAShelf/tuigreet/blob/master/contrib/assets/screenshot-themed.png)

## License

<!-- markdownlint-disable MD059 -->

Following the original source, this project is made available under GNU General
Public License version 3 (GPLv3). See [LICENSE](LICENSE) for more details on the
exact conditions. An online copy is provided
[here](https://www.gnu.org/licenses/gpl-3.0.en.html).

<!-- markdownlint-enable MD059 -->
