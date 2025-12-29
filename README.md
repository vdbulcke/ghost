# Ghost

A [Zellij](https://zellij.dev) plugin for spawning floating command terminal pane.
Basically, it is an interactive version of `zrf` (`function zrf () { zellij run --name "$*" --floating -- zsh -ic "$*";}`).


![Demo](https://raw.githubusercontent.com/vdbulcke/ghost/main/img/ghost.gif)


Since `v0.3.0`, if you define a list of completions from `global_completion` plugin configuration, it will load each lines as a list of commands that you can fuzzy search (using [fuzzy-matcher](https://crates.io/crates/fuzzy-matcher)).

Since `v0.6.1`, if you have the option to cache last used `.ghost` file.

If the plugin finds a `.ghost` at the current Zellij session's working directory, it will add those completions along with the global_completions.

![Completion](./img/fuzzy_search.png)

> NOTE: When autocomplete is toggled off with <kbd>Ctrl</kbd>+<kbd>x</kbd> in the floating pane, you can run any command to run in the cwd in the floating pane instead of fuzzy find commands.

## Requirements

Zellij version `v0.40.0` or later.

| Zellij    | ghost    |
| --------- | -------- |
| `v0.40.x` | `v0.4.0` |
| `v0.41.1` | `v0.5.0` |


### Zellij Plugin Permission

| Permission               | Why                                 |
| -------------------------|-------------------------------------|
| `ReadApplicationState`   | Subscribe to Pane and tab events    |
| `RunCommands`            | Creating Run Command floating pane  |
| `ChangeApplicationState` | Setting plugin pane name            |

### Host Filesystem Access

> NOTE: zellij `v0.39.0` changed the `/host` path on plugin from the current zellij session working dir to the previous pane working dir (see [Release Notes](https://github.com/zellij-org/zellij/releases/tag/v0.39.0) and [related docs](https://zellij.dev/documentation/plugin-api-file-system) for more info)

The plugin will look for a `/host/.ghost` file (i.e. the current working of the previous focussed pane) to load a list of predefined commands (like a bash_history). Additionally, if cache is enabled, it will save a copy of the `.ghost` file in the `/tmp` folder.


Example of a `.ghost` file:
```bash
cargo build
## this is a comment starting with '#'
	# this is also a comment
terraform apply



## empty lines are also ignored
go test -v ./...

```

## Install

### Upgrade

#### Breaking Change

##### v0.4.0

~~* renamed config `cwd` with `exec_cwd`~~

`cwd` and `exec_cwd` are deprecated since `0.6.0` along with `ghost_launcher`.


### Download WASM Binary


* Download `ghost.wasm` binary from [release page](https://github.com/vdbulcke/ghost/releases).
* Verify binary signature with cosign (see instruction bellow)
* copy binary to zellij plugin directory:
     - `mv target/wasm32-wasip1/release/ghost.wasm ~/.config/zellij/plugins/`


#### Validate Signature With Cosign

Make sure you have `cosign` installed locally (see [Cosign Install](https://docs.sigstore.dev/cosign/installation/)).

Then you can use the `./verify_signature.sh` in this repo:

```bash
./verify_signature.sh PATH_TO_DOWNLOADED_ARCHIVE TAG_VERSION
```
for example
```bash
$ ./verify_signature.sh ~/Downloads/ghost.wasm v0.4.0

Checking Signature for version: v0.4.0
Verified OK

```

### Build from source

> WARNING: requires to have rust installed and wasm `rustup target add wasm32-wasip1`

* `git clone git@github.com:vdbulcke/ghost.git`
* `cd ghost`
* `cargo build --release`
* `mv target/wasm32-wasip1/release/ghost.wasm ~/.config/zellij/plugins/`


## Configuration

### Required Configuration

#### Zsh Shell

| Key          | value |
|--------------|------ |
| `shell`      | `zsh` |
| `shell_flag` | `-ic` |

#### fish Shell

| Key          | value  |
|--------------|--------|
| `shell`      | `fish` |
| `shell_flag` | `-c`   |


#### Bash Shell

| Key          | value  |
|--------------|--------|
| `shell`      | `bash` |
| `shell_flag` | `-ic`  |


### Optional Configuration


| Key                                                       | value                      | description                                                |
|-----------------------------------------------------------|----------------------------|------------------------------------------------------------|
| ~~`cwd` (**deprecated** use `exec_cwd` instead)~~         | directory path             | set working dir for command                                |
| ~~`exec_cwd` (zellij `0.40.0`)~~ (deprecated in `v0.6.0`) | directory path             | set working dir for command                                |
| ~~`ghost_launcher`~~ (deprecated in `v0.6.0`)             | GhostLauncher pane name    | plugin will automatically close that pane                  |
| `embedded`                                                | `false`                    | created command panes are embedded instead of floating     |
| `debug`                                                   | `false`                    | display debug info                                         |
| `cache`                                                   | `false`                    | cache last used ghost file (valid only till zellij is open)|
| `global_completion`                                       | multine list of completion | global list of completion to inlude to `/host/.ghost` file |


> NOTE:  The "GhostLauncher" (`ghost_launcher`, `exec_cwd`) is no longer need. Now the `cwd` is derived from the `get_plugin_ids().initial_cwd` plugin api.

## Launch Plugin

```bash
zellij plugin --floating --configuration "shell=zsh,shell_flag=-ic,cwd=$(pwd)" -- "file:$HOME/.config/zellij/plugins/ghost.wasm"
```

## Config Keybindings

```kdl
shared_except "locked" {

    // ghost native plugin (with default zellij cwd)
    bind "Alt (" {
        LaunchOrFocusPlugin "file:~/.config/zellij/plugins/ghost.wasm" {
            floating true

            // Ghost config
            shell "zsh"      // required ("bash", "fish", "zsh")
            shell_flag "-ic" // required ("-ic",  "-c",    "-ic")

            // optional config
            // ghost_launcher "GhostLauncher" // name of the Ghost launcher pane (default GhostLauncher)
            // debug false                    // display debug info, config, parse command etc
            // embedded false                 // spawned command pane will be embedded instead of floating pane
            // cache false                    // cache last used .ghost file to /tmp


            // NOTE: using 'r#"' and '"#' rust like multi string delimeter
            global_completion r#"
                tf apply -auto-approve
                cargo build
                go test -v  ./...

            "#
        }
    }


}
```

## Limitations

### Resizing

- UI column size is not handled, so resizing to plugin window too small may crash the plugin.
- UI row size is partially handled, where it will minimize to a simple prompt if the plugin window becomes too small.


## Note

This my first rust project, so the code might not be the most idiomatic rust.
Inspiration was taken from other [zellij plugins](https://zellij.dev/documentation/plugin-examples).
