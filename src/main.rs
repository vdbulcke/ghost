use ansi_term::{Colour::Fixed, Style};
use fuzzy_matcher::FuzzyMatcher;
use owo_colors::OwoColorize;
use shellwords::MismatchedQuotes;
use zellij_tile::prelude::*;

use fuzzy_matcher::skim::SkimMatcherV2;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::{collections::BTreeMap, path::PathBuf};
// use sprintf::sprintf;

struct State {
    focused_tab_pos: usize,
    launcher_pane_name: String,
    embedded: bool,
    launcher_pane_id: Option<u32>,
    input: String,
    input_cusror_index: usize,
    userspace_configuration: BTreeMap<String, String>,
    completion_enabled: bool,
    completion: Vec<String>,
    completion_match: Option<String>,
    fz_matcher: SkimMatcherV2,
}

impl Default for State {
    fn default() -> Self {
        Self {
            launcher_pane_name: String::from("GhostLauncher"),
            launcher_pane_id: None,
            userspace_configuration: BTreeMap::default(),
            focused_tab_pos: 0,
            embedded: false,
            input: String::default(),
            input_cusror_index: 0,
            completion_enabled: false,
            completion: Vec::default(),
            completion_match: None,
            fz_matcher: SkimMatcherV2::default(),
        }
    }
}
/// get the focused tab position
fn get_focused_tab(tab_infos: &Vec<TabInfo>) -> Option<usize> {
    for t in tab_infos {
        if t.active {
            return Some(t.position.clone());
        }
    }
    None
}

impl State {
    /// get the launcher pane by title or none
    fn get_ghost_launcher_pane(&self, pane_manifest: &PaneManifest) -> Option<u32> {
        let panes = pane_manifest.panes.get(&self.focused_tab_pos);
        if let Some(panes) = panes {
            for pane in panes {
                if !pane.is_plugin && pane.title == self.launcher_pane_name {
                    return Some(pane.id.clone());
                }
            }
        }
        None
    }

    /// close current plugins and its hepler pane
    fn close(&self) {
        if self.launcher_pane_id.is_some() {
            close_terminal_pane(self.launcher_pane_id.unwrap());
        }
        close_plugin_pane(get_plugin_ids().plugin_id);
    }

    fn fuzzy_find_completion(&mut self) {
        let mut best_score = 0;

        // reset match
        self.completion_match = None;
        for l in self.completion.iter() {
            if let Some(score) = self.fz_matcher.fuzzy_match(l, &self.input) {
                if score > best_score {
                    best_score = score;
                    self.completion_match = Some(l.to_string());
                }
            }
        }
    }

    /// remove_input_at_index  removes char at the
    /// cursor index and update input.
    /// Returns true if the input has change
    fn remove_input_at_index(&mut self) -> bool {
        if self.input.is_empty() {
            self.input.pop();
        } else if self.input_cusror_index > 0 && self.input_cusror_index <= self.input.len() {
            self.input.remove(self.input_cusror_index - 1);
            // update cursor index
            self.input_cusror_index -= 1;

            return true;
        } else if self.input_cusror_index == 0 {
            self.input.remove(0);
        }
        return false;
    }

    /// remove_input_at_index  removes char at the
    /// cursor index and update input.
    /// Returns true if the input has change
    fn insert_input_at_index(&mut self, c: char) -> bool {
        if self.input.is_empty() {
            self.input.push(c);

            // update cursor index
            self.input_cusror_index += 1;
        } else if self.input_cusror_index > 0 && self.input_cusror_index <= self.input.len() {
            self.input.insert(self.input_cusror_index, c);
            // update cursor index
            self.input_cusror_index += 1;

            return true;
        } else if self.input_cusror_index == 0 {
            self.input.insert(0, c);
            self.input_cusror_index += 1;
        }
        return false;
    }

    /// print the input prompt
    fn print_prompt(&self, _rows: usize, _cols: usize) {
        let mut prompt = " $ ".cyan().bold().to_string();
        // if not enough space in UI
        // input prompt
        if self.completion_enabled {
            prompt = " > ".cyan().bold().to_string();
        }
        if self.input.is_empty() {
            if self.completion_enabled {
                println!(
                    "{} {}{}",
                    prompt,
                    "┃".bold().white(),
                    "Fuzzy find command".dimmed().italic().to_string(),
                );
            } else {
                println!(
                    "{} {}{}",
                    prompt,
                    "┃".bold().white(),
                    "Type command to run".dimmed().italic().to_string(),
                );
            }
        } else {
            self.print_non_empty_input_prompt(prompt);
        }
    }

    fn print_non_empty_input_prompt(&self, prompt: String) {
        if self.input_cusror_index == self.input.len() {
            println!(
                "{} {}{}",
                prompt,
                self.input.dimmed().to_string(),
                "┃".bold().white(),
            );
        } else if self.input_cusror_index < self.input.len() {
            let copy = self.input.clone();
            let (before_curs, after_curs) = copy.split_at(self.input_cusror_index);

            println!(
                "{} {}{}{}",
                prompt,
                before_curs.dimmed().to_string(),
                "┃".bold().white(),
                after_curs.dimmed().to_string()
            );
        }
    }

    fn check_valid_cmd(&self) -> Result<Vec<String>, MismatchedQuotes> {
        if self.completion_enabled {
            if let Some(cmd) = &self.completion_match {
                return shellwords::split(cmd);
            }
        }
        shellwords::split(&self.input)
    }

    /// Create a RunCommand pane with input_cmd if valid
    fn run_command(&mut self, input_cmd: String) {
        // get working dir from config
        let plugin_cwd = self.userspace_configuration.get("cwd");
        let cwd = match plugin_cwd {
            Some(path) => {
                let mut pb = PathBuf::new();
                pb.push(path);
                Some(pb)
            }
            _ => None,
        };

        // parse command + params and validate shell compliant
        let command = match shellwords::split(&input_cmd) {
            Ok(cmd) => Some(cmd),
            Err(_) => None,
        };

        if let Some(_) = command {
            // get the shell args from config
            if let Some(shell) = self.userspace_configuration.get("shell") {
                if let Some(shell_flag) = self.userspace_configuration.get("shell_flag") {
                    // e.g. "zsh"
                    let zsh_cmd = shell.to_string();
                    let mut exec = PathBuf::new();
                    exec.push(zsh_cmd);
                    let mut zsh_args = Vec::new();
                    // e.g. "-ic"
                    zsh_args.push(shell_flag.to_owned());
                    zsh_args.push(input_cmd.to_owned());

                    if self.embedded {
                        open_command_pane(CommandToRun {
                            path: exec,
                            args: zsh_args,
                            cwd,
                        });
                    } else {
                        open_command_pane_floating(CommandToRun {
                            path: exec,
                            args: zsh_args,
                            cwd,
                        });
                    }
                    if self.launcher_pane_id.is_some() {
                        close_terminal_pane(self.launcher_pane_id.unwrap());
                    }
                    self.input = String::default();
                    close_plugin_pane(get_plugin_ids().plugin_id);
                }
            }
        }
    }
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.userspace_configuration = configuration;

        // override default launcher pane name if config exists
        if let Some(spwaner_pane_name) = self.userspace_configuration.get("ghost_launcher") {
            self.launcher_pane_name = spwaner_pane_name.clone();
        }

        // use embedded pane instead of floating if config exists
        if let Some(embedded) = self.userspace_configuration.get("embedded") {
            if embedded == "true" {
                self.embedded = true;
            }
        }

        // Permission
        // - ReadApplicationState => for Tab and Pane update
        // - RunCommands => to run floating command terminal
        // - ChangeApplicationState => rename plugin pane, close managed paned
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::RunCommands,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[
            EventType::ModeUpdate,
            EventType::TabUpdate,
            EventType::PaneUpdate,
            EventType::Key,
        ]);

        // File .ghost must exist in the current path (zellij cwd dir is mounted as /host)
        // NOTE: /host is the cwd of where the zellij session started
        //       and not the current cwd of the pane itself
        let filename = "/host/.ghost".to_owned();
        if let Ok(lines) = read_lines(filename) {
            // Consumes the iterator, returns an (Optional) String
            for line in lines {
                if let Ok(cmd) = line {
                    // ignore commented lines starting with '#'
                    // or empty line
                    if !cmd.trim_start().starts_with("#") && !cmd.trim_start().is_empty() {
                        if !self.completion_enabled {
                            self.completion_enabled = true;
                        }
                        self.completion.push(cmd);
                    }
                }
            }
        }

        rename_plugin_pane(get_plugin_ids().plugin_id, "Ghost");
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::TabUpdate(tab_info) => {
                // keep track of focused to tab in order
                // to find panes in active tab
                if let Some(position) = get_focused_tab(&tab_info) {
                    self.focused_tab_pos = position;
                }
                should_render = true;
            }
            Event::PaneUpdate(pane_manifest) => {
                // keep track of pane id of the launcher pane
                // in order to close it later
                self.launcher_pane_id = self.get_ghost_launcher_pane(&pane_manifest);
                if self.launcher_pane_id.is_some() {
                    close_terminal_pane(self.launcher_pane_id.unwrap());
                }
                should_render = true;
            }
            Event::Key(Key::Char('\n')) => {
                if self.completion_enabled {
                    if let Some(cmd) = &self.completion_match {
                        // run completion match
                        self.run_command(cmd.to_owned());
                    }
                } else {
                    // if completion disable run intput as command
                    self.run_command(self.input.to_owned());
                }
            }
            Event::Key(Key::Backspace) => {
                if self.remove_input_at_index() {
                    // update fuzzy find result
                    self.fuzzy_find_completion();
                }
                should_render = true;
            }
            Event::Key(Key::Char(c)) => {
                if self.insert_input_at_index(c) {
                    self.fuzzy_find_completion();
                }
                should_render = true;
            }
            Event::Key(Key::Left) => {
                if self.input_cusror_index > 0 {
                    self.input_cusror_index -= 1;
                }
                should_render = true;
            }
            Event::Key(Key::Right) => {
                if self.input_cusror_index < self.input.len() {
                    self.input_cusror_index += 1;
                }
                should_render = true;
            }
            Event::Key(Key::Esc | Key::Ctrl('c')) => {
                self.close();
                should_render = true;
            }
            Event::Key(Key::Ctrl('x')) => {
                self.completion_enabled = !self.completion_enabled;
                should_render = true;
            }
            _ => (),
        };

        should_render
    }

    fn render(&mut self, rows: usize, cols: usize) {
        // get the shell args from config
        if self.userspace_configuration.get("shell").is_none() {
            if self.userspace_configuration.get("shell_flag").is_none() {
                println!("{}", color_bold(RED, "Error 'shell' (zsh|fish|bash)  and 'shell_flag' (e.g '-ic') are required configuration"));
                return;
            }
        }

        let debug = self.userspace_configuration.get("debug");
        // count keep tracks of lines printed
        // 4 lines for CWD and keybinding views
        let mut count = 4;

        // validation info view
        let res = self.check_valid_cmd();
        match res {
            Ok(_) => {
                println!("");
            }
            Err(_) => println!("{}", color_bold(RED, "Invalid Command")),
        }
        count += 1;

        // prompt view
        if rows < 5 {
            // disable competion
            self.completion_enabled = false;
            self.print_prompt(rows, cols);
            return; // no more UI
        }
        if rows < 10 {
            // disable competion
            self.completion_enabled = false;
        }

        self.print_prompt(rows, cols);
        count += 1;

        // completion fuzzy finder
        if self.completion_enabled {
            if let Some(m) = &self.completion_match {
                println!(" $ {}", m);
                println!("");

                count += 2;
            } else {
                println!(" $ {}", "Matched command".dimmed().to_string());
                println!("");
                count += 2;
            }
            println!(" Available completion: ");

            count += 1;
            for l in self.completion.iter() {
                if let Some(_) = self.fz_matcher.fuzzy_match(l, &self.input) {
                    // limits display of completion
                    // based on available rows in pane
                    // with arbitrary buffer for safety
                    if count >= rows - 4 {
                        println!(" - {}", "...".dimmed().to_string());
                        break;
                    }
                    println!(" - {}", l.dimmed().to_string());
                    count += 1;
                }
            }
        }

        // current dir view
        if let Some(plugin_cwd) = self.userspace_configuration.get("cwd") {
            println!("");
            println!(
                " {}: {}",
                color_bold(WHITE, "cwd"),
                plugin_cwd.blue().bold()
            );
        }

        // Key binding view
        println!("");
        println!(
            "  <{}> <{}> Close Plugin <{}> Toggle Completion on/off",
            color_bold(WHITE, "Esc"),
            color_bold(WHITE, "Ctrl+c"),
            color_bold(WHITE, "Ctrl x"),
        );

        if debug.is_some_and(|x| x == "true") {
            println!("input: {}", self.input.to_string());

            println!("Cursor: {}", self.input_cusror_index);
            println!("len: {}", self.input.len());

            println!(
                "{} {:#?}",
                color_bold(GREEN, "Runtime configuration:"),
                self.userspace_configuration
            );
        }
    }
}

pub const CYAN: u8 = 51;
pub const GRAY_LIGHT: u8 = 238;
pub const GRAY_DARK: u8 = 245;
pub const WHITE: u8 = 15;
pub const BLACK: u8 = 16;
pub const RED: u8 = 124;
pub const GREEN: u8 = 154;
pub const ORANGE: u8 = 166;

fn color_bold(color: u8, text: &str) -> String {
    format!("{}", Style::new().fg(Fixed(color)).bold().paint(text))
}

// src: https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
