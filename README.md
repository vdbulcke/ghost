# Ghost

A [Zellij](https://zellij.dev) plugin for spawning floating command terminal pane.
Basically, it is an interactive version of `zrf` (`function zrf () { zellij run --name "$*" --floating -- zsh -ic "$*";}`).


![Demo](https://raw.githubusercontent.com/vdbulcke/ghost/main/img/ghost.gif)

If the plugin finds a `.ghost` at the working of where you started your zellij session, it will load each lines as a list of commands that you can fuzzy search (using [fuzzy-matcher](https://crates.io/crates/fuzzy-matcher)).

![Completion](./img/fuzzy_search.png)


## Requirements

Zellij version `v0.38.0` or later.

### Zellij Plugin Permission 

| Permission               | Why                                 |
| -------------------------|-------------------------------------|
| `ReadApplicationState`   | Subscribe to Pane and tab events    |
| `RunCommands`            | Creating Run Command floating pane  | 
| `ChangeApplicationState` | Setting plugin pane name            |

### Host Filesystem Access

[Zellij maps the folder where Zellij was started](https://zellij.dev/documentation/plugin-api-file-system) to `/host` path on the plugin (e.g. your home dir or `default_cwd` in your zellij or the current dir where you started your zellij session).

The plugin will look for a `/host/.ghost` file (i.e. at the root of the dir of you current zellij session) to load a list of predefined commands (like a bash_history).


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

### Download WASM Binary


* Download `ghost.wasm` binary from [release page](https://github.com/vdbulcke/ghost/releases).
* Verify binary signature with cosign (see instruction bellow)
* copy binary to zellij plugin dir: 
     - `mv target/wasm32-wasi/release/ghost.wasm ~/.config/zellij/plugins/`


#### Validate Signature With Cosign

Make sure you have `cosign` installed locally (see [Cosign Install](https://docs.sigstore.dev/cosign/installation/)).

Then you can use the `./verify_signature.sh` in this repo: 

```bash
./verify_signature.sh PATH_TO_DOWNLOADED_ARCHIVE TAG_VERSION
```
for example
```bash
$ ./verify_signature.sh ~/Downloads/ghost.wasm v0.1.0

Checking Signature for version: v0.1.0
Verified OK

```





### Build from source

> WARNING: requires to have rust installed and wasm `rustup target add wasm32-wasi`

* `git clone git@github.com:vdbulcke/ghost.git`
* `cd ghost`
* `cargo build --release`
* `mv target/wasm32-wasi/release/ghost.wasm ~/.config/zellij/plugins/`


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


| Key              | value                   | desctiption                                            |
|------------------|-------------------------|--------------------------------------------------------|
| `cwd`            | directory path          | set working dir for command                            |
| `embedded`       | `true`                  | created command panes are embedded instead of floating |
| `ghost_launcher` | GhostLauncher pane name | plugin will automatically close that pane              |
| `debug`          | `true`                  | display debug info                                     |




## Launch Plugin

```bash
zellij action launch-or-focus-plugin --floating --configuration "shell=zsh,shell_flag=-ic,cwd=$(pwd)" "file:$HOME/.config/zellij/plugins/ghost.wasm"
```

## Config Keybindings

> NOTE: The `LaunchOrFocusPlugin` keybing action does not allow to dynamically pass the cwd to the plugin. As a workaround, you can use the `Run` keybinding action to execute the `zellij action launch-or-focus-plugin` from a RunCommand pane where you can pass the plugin config `cwd=$(pwd)`. The cwd should be the same as the previously focused pane.

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
        }
    }

    // using GhostLauncher "hack" to pass the cwd=$(pwd) as runtime config 
    bind "Alt )" {
         // this 
         Run "bash" "-ic" "zellij action launch-or-focus-plugin --floating --configuration \"shell=zsh,shell_flag=-ic,cwd=$(pwd),ghost_launcher=GhostLauncher,debug=false\" \"file:$HOME/.config/zellij/plugins/ghost.wasm\"" {
            floating true
            name "GhostLauncher" // this must match ghost_launcher=GhostLauncher 
                                 // the plugin will automatically close the pane
                                 // with title "GhostLauncher"
        }
    }
}
```

## Limitations

### resizing

UI column size is not handled, so resizing to plugin window too small may crash the plugin. 
UI row size is partiallt handled, where it will minimize to a simple prompt if the plugin window becomes too small.


## Note

This my first rust project, so the code might not be the most idiomatic rust.
Inpiration was taken from other [zellij plugins](https://zellij.dev/documentation/plugin-examples).
