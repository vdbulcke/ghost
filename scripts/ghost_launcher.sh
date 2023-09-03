#!/bin/bash


## pass the cwd dynamically 
zellij action launch-or-focus-plugin --floating --configuration "shell=zsh,shell_flag=-ic,cwd=$(pwd)" "file:$HOME/.config/zellij/plugins/ghost.wasm"