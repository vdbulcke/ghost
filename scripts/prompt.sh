#!/bin/bash


function zrf () { 
  zellij run --name "$*" --floating -- zsh -ic "$*";
}


function prompt() {
  echo ""
  zrf $(gum input --width=200 --char-limit=200 )
}

function completion() {
  
  zrf $(cat ~/.ghost |  grep -v '#' | fzf   --height 4)
}


gum confirm --affirmative="Prompt"  --negative="Completions"  --unselected.foreground="#0096FF" "Select option" && prompt || completion

