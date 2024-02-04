use zellij_tile::prelude::*;
use std::collections::{BTreeMap, HashMap};


#[derive(Default)]
struct VimAutolock {
    current_tab: Option<usize>,
    last_active_process: HashMap<usize, String>,
    was_locked: bool,
    current_mode: InputMode,
}

register_plugin!(VimAutolock);

impl ZellijPlugin for VimAutolock {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[
            EventType::ModeUpdate,
            EventType::TabUpdate,
            EventType::PaneUpdate,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::ModeUpdate(mode_info) => {
                self.current_mode = mode_info.mode;
                if mode_info.mode != InputMode::Normal && mode_info.mode != InputMode::Locked {
                    self.was_locked = false;
                    return false;
                } else if self.was_locked {
                    return false;
                }

                let Some(tab) = self.current_tab else { return false; };
                let Some(proc) = self.last_active_process.get(&tab) else {
                    return false;
                };
                if proc.ends_with("vim") {
                    switch_to_input_mode(&InputMode::Locked);
                    self.was_locked = true;
                }
            }
            Event::TabUpdate(tab_info) => {
                self.current_tab = get_focused_tab(&tab_info).map(|tab| tab.position);
            }
            Event::PaneUpdate(pane_info) => {
                let Some(tab) = self.current_tab else { return false; };
                let Some(current_pane) = get_focused_pane(tab, &pane_info) else { return false; };
                let Some(proc) = current_pane.title.split_whitespace().nth(0) else { return false; };

                if proc.ends_with("vim") && self.current_mode == InputMode::Normal {
                    switch_to_input_mode(&InputMode::Locked);
                    self.was_locked = true;
                } else if !proc.ends_with("vim") {
                    self.was_locked = false;
                }
                self.last_active_process.insert(tab, proc.to_owned());
            }
            _ => (),
        };

        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) { }
}
