use ansi_term::{Colour::Fixed, Style};
use owo_colors::OwoColorize;
use zellij_tile::prelude::*;

use std::{collections::BTreeMap, path::PathBuf};
// use sprintf::sprintf;

struct State {
    focused_tab_pos: usize,
    launcher_pane_name: String,
    embedded: bool,
    launcher_pane_id: Option<u32>,
    command: String,
    userspace_configuration: BTreeMap<String, String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            launcher_pane_name: String::from("GhostLauncher"),
            launcher_pane_id: None,
            userspace_configuration: BTreeMap::default(),
            focused_tab_pos: 0,
            embedded: false,
            command: String::default(),
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
                let command = match shellwords::split(self.command.as_str()) {
                    Ok(cmd) => Some(cmd),
                    Err(_) => None,
                };

                if let Some(_) = command {
                    // get the shell args from config
                    if let Some(shell) = self.userspace_configuration.get("shell") {
                        if let Some(shell_flag) = self.userspace_configuration.get("shell_flag") {
                            // }
                            // if shell.is_some() && shell_flag.is_some() {
                            // e.g. "zsh"
                            let zsh_cmd = shell.to_string();
                            let mut exec = PathBuf::new();
                            exec.push(zsh_cmd);
                            let mut zsh_args = Vec::new();
                            // e.g. "-ic"
                            zsh_args.push(shell_flag.to_owned());
                            zsh_args.push(self.command.to_owned());

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
                            self.command = String::default();
                            if self.launcher_pane_id.is_some() {
                                close_terminal_pane(self.launcher_pane_id.unwrap());
                            }
                            self.command = String::default();
                            close_plugin_pane(get_plugin_ids().plugin_id);
                        }
                    }
                }
            }
            Event::Key(Key::Backspace) => {
                self.command.pop();

                should_render = true;
            }
            Event::Key(Key::Char(c)) => {
                self.command.push(c);

                should_render = true;
            }
            Event::Key(Key::Esc | Key::Ctrl('c')) => {
                self.close();
                should_render = true;
            }
            // Event::Key(Key::Ctrl('x')) => {
            //     if self.launcher_pane_id.is_some() {
            //         close_terminal_pane(self.launcher_pane_id.unwrap());
            //     }
            //     should_render = true;
            // }
            _ => (),
        };

        should_render
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        println!("");
        println!(
            "{} {}",
            " > ".cyan().bold(),
            if self.command.is_empty() {
                "Type command to run".dimmed().italic().to_string()
            } else {
                self.command.dimmed().italic().to_string()
            }
        );

        if let Some(plugin_cwd) = self.userspace_configuration.get("cwd") {
            println!("");
            println!(
                " {}: {}",
                color_bold(WHITE, "cwd"),
                plugin_cwd.blue().bold()
            );
        }

        // get the shell args from config
        if self.userspace_configuration.get("shell").is_none() {
            if self.userspace_configuration.get("shell_flag").is_none() {
                println!("{}", color_bold(RED, "Error 'shell' (zsh|fish|bash)  and 'shell_flag' (e.g '-ic') are required configuration"));
            }
        }
        let debug = self.userspace_configuration.get("debug");

        let res = shellwords::split(self.command.as_str());
        match res {
            Ok(p) => {
                let cmd = p.first();

                if debug.is_some_and(|x| x == "true") {
                    println!("{}", color_bold(GREEN, "Parsed Command"));
                    if cmd.is_some() {
                        println!("cmd: {}", p.first().unwrap());
                    }
                    println!("param: {:#?}", p);
                }
            }
            Err(_) => println!("{}", color_bold(RED, "Invalid Command")),
        }
        if debug.is_some_and(|x| x == "true") {
            println!(
                "{} {:#?}",
                color_bold(GREEN, "Runtime configuration:"),
                self.userspace_configuration
            );
        }
        println!("");
        println!(
            "  <{}> <{}> Close Plugin ",
            color_bold(WHITE, "Esc"),
            color_bold(WHITE, "Ctrl+c"),
        );
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
