# Ghost

A [Zellij](https://zellij.dev) plugin for spawning floating command terminal pane.
Basically, it is an interactive version of `zrf` (`function zrf () { zellij run --name "$*" --floating -- zsh -ic "$*";}`).


![Demo](https://raw.githubusercontent.com/vdbulcke/ghost/main/img/ghost.gif)

## Requirements

Zellij version `v0.38.0` or later.

### Zellij Plugin Permission 

| Permission               | Why                                 |
| -------------------------|-------------------------------------|
| `ReadApplicationState`   | Subscribe to Pane and tab events    |
| `RunCommands`            | Creating Run Command floating pane  | 
| `ChangeApplicationState` | Setting plugin pane name            |




## Install

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

## Keybinding

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


## Note

This my first rust project, so the code might not be the most idiomatic rust.

