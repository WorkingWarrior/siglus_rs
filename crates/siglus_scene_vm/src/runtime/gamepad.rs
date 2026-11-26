//! Player-facing VN actions, separate from the scripts' raw joypad button table.

#[cfg(test)]
use super::globals::SyscomPendingProcKind;
use super::CommandContext;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GamepadAction {
    Cancel,
    Auto,
    Backlog,
    HoldForceSkip,
    ToggleSkip,
    Menu,
    QuickSave,
    Unassigned,
}

impl CommandContext {
    fn gamepad_reading_controls_enabled(&self) -> bool {
        !self.platform_shortcuts_blocked()
            && !self.excall_state.ex_call_flag
            && !self.wait.button_selection_waiting()
            && self.globals.focused_stage_mwnd.is_none()
            && !self.wait.movie_waiting()
            && !self.globals.mov.playing
            && (self.wait.message_key_waiting()
                || self.wait.message_reveal_waiting()
                || self.ui.message_text().is_some()
                || self.globals.stage_forms.values().any(|stage| {
                    stage
                        .mwnd_lists
                        .values()
                        .flatten()
                        .any(|m| m.open && m.selection.is_none() && !m.msg_text.is_empty())
                }))
    }

    pub(super) fn gamepad_skip_active(&self) -> bool {
        self.gamepad_skip_held
            && self.gamepad_reading_controls_enabled()
            && !self.globals.syscom.hide_mwnd.onoff
            && !self.globals.script.ctrl_disable
    }

    pub(crate) fn on_gamepad_action(&mut self, action: GamepadAction, down: bool, cancelled: bool) {
        if action == GamepadAction::Unassigned {
            return;
        }
        // Releases must be honored even after a menu or focus change. They
        // never toggle a feature or manufacture a decide/cancel edge.
        if action == GamepadAction::HoldForceSkip {
            self.gamepad_skip_held = down
                && !cancelled
                && self.gamepad_reading_controls_enabled()
                && !self.globals.script.skip_disable
                && !self.globals.script.ctrl_disable;
            if self.gamepad_skip_held {
                self.globals.syscom.auto_mode.onoff = false;
            }
            return;
        }
        if !down || cancelled || self.globals.system.messagebox_modal.is_some() {
            return;
        }
        if action == GamepadAction::Cancel {
            self.gamepad_cancel();
            return;
        }
        if action == GamepadAction::Backlog && self.globals.syscom.msg_back_open {
            self.close_msg_back_proc();
            return;
        }
        if action == GamepadAction::Menu {
            if self.globals.syscom.msg_back_open {
                self.close_msg_back_proc();
            } else if self.excall_state.ex_call_flag || self.globals.syscom.menu_open {
                // The game's own menu script owns its cancel key.
                self.gamepad_cancel();
            } else if self.gamepad_reading_controls_enabled() {
                self.open_syscom_menu_from_cancel_key();
            }
            return;
        }
        if !self.gamepad_reading_controls_enabled() {
            return;
        }
        match action {
            GamepadAction::Auto if self.globals.syscom.auto_mode.check_enabled() != 0 => {
                self.globals.syscom.auto_mode.onoff = !self.globals.syscom.auto_mode.onoff;
                if self.globals.syscom.auto_mode.onoff {
                    self.globals.syscom.read_skip.onoff = false;
                    self.gamepad_skip_held = false;
                }
            }
            GamepadAction::Backlog => self.open_msg_back_proc(),
            GamepadAction::ToggleSkip
                if !self.globals.script.skip_disable && !self.globals.script.ctrl_disable =>
            {
                let enabled = !self.globals.syscom.read_skip.onoff;
                if !enabled || self.runtime_read_skip_is_enable() {
                    self.globals.syscom.read_skip.onoff = enabled;
                    self.cant_auto_skip_before_click = !enabled;
                    if enabled {
                        self.globals.syscom.auto_mode.onoff = false;
                    }
                }
            }
            GamepadAction::QuickSave => {
                super::forms::syscom::request_gamepad_quick_save(self);
            }
            _ => {}
        }
    }

    fn gamepad_cancel(&mut self) {
        if self.globals.syscom.msg_back_open {
            self.close_msg_back_proc();
            return;
        }
        if self.globals.focused_editbox.is_some() {
            return;
        }
        if self.globals.selbtn.started || self.globals.focused_stage_mwnd.is_some() {
            self.on_key_down(super::input::VmKey::Escape);
            self.on_key_up(super::input::VmKey::Escape);
            return;
        }
        if !self.excall_state.ex_call_flag
            && !self.globals.syscom.menu_open
            && self.globals.syscom.hide_mwnd.onoff
        {
            self.globals.syscom.hide_mwnd.onoff = false;
            return;
        }
        if self.gamepad_reading_controls_enabled() {
            if !self.globals.script.hide_mwnd_disable
                && self.globals.syscom.hide_mwnd.check_enabled() != 0
            {
                self.globals.syscom.hide_mwnd.onoff = true;
            }
            return;
        }
        if !self.excall_state.ex_call_flag
            && !self.globals.syscom.menu_open
            && !self.wait.movie_waiting()
            && !self.globals.mov.playing
        {
            return;
        }
        // Scripted Siglus menus commonly poll the right-click stock directly;
        // Escape alone need not be a menu's back action.
        self.on_mouse_down(super::input::VmMouseButton::Right);
        self.on_mouse_up(super::input::VmMouseButton::Right);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn reading_context() -> CommandContext {
        let mut ctx = CommandContext::new(PathBuf::from("."));
        ctx.wait.wait_message_reveal_then_key();
        ctx
    }

    #[test]
    fn stage_message_window_enables_controls_without_fallback_ui_text() {
        let mut ctx = CommandContext::new(PathBuf::from("."));
        let stage = ctx
            .globals
            .stage_forms
            .entry(ctx.ids.form_global_stage)
            .or_default();
        stage.ensure_mwnd_list(0, 1);
        let message = &mut stage.mwnd_lists.get_mut(&0).unwrap()[0];
        message.open = true;
        message.msg_text = "A stage-rendered dialogue".into();
        assert!(ctx.ui.message_text().is_none());
        ctx.on_gamepad_action(GamepadAction::Auto, true, false);
        assert!(ctx.globals.syscom.auto_mode.onoff);
        ctx.on_gamepad_action(GamepadAction::Cancel, true, false);
        assert!(ctx.globals.syscom.hide_mwnd.onoff);
        ctx.on_gamepad_action(GamepadAction::Cancel, true, false);
        assert!(!ctx.globals.syscom.hide_mwnd.onoff);
    }

    #[test]
    fn reading_controls_wait_for_foreground_movies_but_allow_dialogue() {
        let mut ctx = reading_context();
        ctx.wait.wait_global_movie(false, false);
        ctx.on_gamepad_action(GamepadAction::Auto, true, false);
        assert!(!ctx.globals.syscom.auto_mode.onoff);
        ctx.wait.clear();
        ctx.wait
            .wait_object_movie(ctx.ids.form_global_stage, 0, 0, false, false);
        ctx.on_gamepad_action(GamepadAction::QuickSave, true, false);
        assert!(ctx.globals.syscom.pending_proc.is_none());
        ctx.wait.clear();
        ctx.wait.wait_message_reveal_then_key();
        ctx.on_gamepad_action(GamepadAction::Auto, true, false);
        assert!(ctx.globals.syscom.auto_mode.onoff);
        ctx.on_gamepad_action(GamepadAction::Menu, true, false);
        assert_eq!(
            ctx.globals.syscom.pending_proc.as_ref().unwrap().kind,
            SyscomPendingProcKind::OpenSyscomMenu
        );
    }

    #[test]
    fn cancel_uses_scripted_right_click_and_restores_hidden_text() {
        let mut ctx = reading_context();
        ctx.excall_state.ex_call_flag = true;
        ctx.on_gamepad_action(GamepadAction::Cancel, true, false);
        assert!(ctx.input.vk_down_up_stock(0x02));
        assert!(!ctx.input.vk_is_down(0x02));
        assert!(
            ctx.globals.syscom.pending_proc.is_none(),
            "no recursive menu"
        );
        ctx.input.clear_all();
        ctx.excall_state.ex_call_flag = false;
        ctx.globals.syscom.msg_back_open = true;
        ctx.on_gamepad_action(GamepadAction::Cancel, true, false);
        assert!(!ctx.globals.syscom.msg_back_open);
        assert!(
            ctx.globals.syscom.pending_proc.is_none(),
            "closing history must not open a menu"
        );
        assert!(!ctx.input.vk_down_up_stock(0x02));
        ctx.globals.syscom.hide_mwnd.onoff = true;
        ctx.on_gamepad_action(GamepadAction::Cancel, true, false);
        assert!(!ctx.globals.syscom.hide_mwnd.onoff);
        assert!(!ctx.input.vk_down_up_stock(0x02));
    }

    #[test]
    fn auto_toggles_once_and_honors_feature_disable() {
        let mut ctx = reading_context();
        ctx.on_gamepad_action(GamepadAction::Auto, true, false);
        assert!(ctx.globals.syscom.auto_mode.onoff);
        ctx.on_gamepad_action(GamepadAction::Auto, false, false);
        assert!(ctx.globals.syscom.auto_mode.onoff);
        ctx.on_gamepad_action(GamepadAction::Auto, true, false);
        assert!(!ctx.globals.syscom.auto_mode.onoff);
        ctx.globals.syscom.auto_mode.enable = false;
        ctx.on_gamepad_action(GamepadAction::Auto, true, false);
        assert!(!ctx.globals.syscom.auto_mode.onoff);
    }

    #[test]
    fn force_skip_includes_unread_text_but_obeys_script_locks_and_cancel() {
        let mut ctx = reading_context();
        ctx.on_gamepad_action(GamepadAction::HoldForceSkip, true, false);
        assert!(!ctx.read_skip_enable_flag);
        assert!(
            ctx.runtime_is_skipping_msg(),
            "LB force-skip includes unread text"
        );
        ctx.globals.script.skip_disable = true;
        assert!(!ctx.runtime_is_skipping_msg());
        ctx.globals.script.skip_disable = false;
        ctx.on_gamepad_action(GamepadAction::HoldForceSkip, false, true);
        assert!(!ctx.runtime_is_skipping_msg());
        assert!(!ctx.input.vk_is_down(0x11));
    }

    #[test]
    fn shortcuts_do_not_run_in_menus_choices_or_editboxes() {
        let mut ctx = reading_context();
        ctx.excall_state.ex_call_flag = true;
        for action in [GamepadAction::Auto, GamepadAction::QuickSave] {
            ctx.on_gamepad_action(action, true, false);
        }
        assert!(!ctx.globals.syscom.auto_mode.onoff);
        assert!(ctx.globals.syscom.pending_proc.is_none());
        ctx.excall_state.ex_call_flag = false;
        ctx.wait.wait_group_selection(1, 1, 0);
        ctx.on_gamepad_action(GamepadAction::HoldForceSkip, true, false);
        assert!(!ctx.gamepad_skip_held);
        ctx.wait.clear();
        ctx.globals.focused_editbox = Some((1, 0));
        ctx.on_gamepad_action(GamepadAction::Auto, true, false);
        assert!(!ctx.globals.syscom.auto_mode.onoff);
    }

    #[test]
    fn quick_save_queues_one_quick_slot_and_honors_save_disable() {
        let mut ctx = reading_context();
        ctx.pending_auto_savepoint = true;
        ctx.on_gamepad_action(GamepadAction::QuickSave, true, false);
        let pending = ctx.globals.syscom.pending_proc.take().unwrap();
        assert_eq!(pending.kind, SyscomPendingProcKind::QuickSave);
        assert_eq!(pending.save_id, 0);
        assert!(!pending.warning);
        assert!(
            ctx.take_runtime_save_request().is_none(),
            "host consumes the queued request"
        );
        ctx.on_gamepad_action(GamepadAction::QuickSave, false, false);
        assert!(ctx.globals.syscom.pending_proc.is_none());
        ctx.globals.syscom.save_feature.enable = false;
        ctx.on_gamepad_action(GamepadAction::QuickSave, true, false);
        assert!(ctx.globals.syscom.pending_proc.is_none());
    }

    #[test]
    fn toggle_skip_still_protects_unread_text() {
        let mut ctx = reading_context();
        ctx.on_gamepad_action(GamepadAction::ToggleSkip, true, false);
        assert!(!ctx.globals.syscom.read_skip.onoff);
        ctx.read_skip_enable_flag = true;
        ctx.on_gamepad_action(GamepadAction::ToggleSkip, true, false);
        assert!(ctx.globals.syscom.read_skip.onoff);
    }

    #[test]
    fn hidden_text_can_be_restored_and_disabled_features_stay_disabled() {
        let mut ctx = reading_context();
        ctx.on_gamepad_action(GamepadAction::Cancel, true, false);
        assert!(ctx.globals.syscom.hide_mwnd.onoff);
        ctx.on_gamepad_action(GamepadAction::Cancel, true, false);
        assert!(!ctx.globals.syscom.hide_mwnd.onoff);
        ctx.globals.script.hide_mwnd_disable = true;
        ctx.on_gamepad_action(GamepadAction::Cancel, true, false);
        assert!(!ctx.globals.syscom.hide_mwnd.onoff);
    }

    #[test]
    fn force_skip_stops_immediately_on_dynamic_ctrl_disable() {
        let mut ctx = reading_context();
        ctx.on_gamepad_action(GamepadAction::HoldForceSkip, true, false);
        assert!(ctx.runtime_is_skipping_msg());
        ctx.globals.script.ctrl_disable = true;
        assert!(!ctx.runtime_is_skipping_msg());
        assert!(!ctx.runtime_is_skipping());
        ctx.on_gamepad_action(GamepadAction::HoldForceSkip, false, false);
        assert!(!ctx.gamepad_skip_held);
        ctx.globals.script.ctrl_disable = false;
        assert!(!ctx.runtime_is_skipping_msg());
        ctx.globals.script.skip_disable = true;
        ctx.on_gamepad_action(GamepadAction::HoldForceSkip, true, false);
        assert!(!ctx.runtime_is_skipping_msg());
    }

    #[test]
    fn cancel_outside_dialogue_and_excall_does_not_open_syscom_menu() {
        let mut ctx = CommandContext::new(PathBuf::from("."));
        ctx.on_gamepad_action(GamepadAction::Cancel, true, false);
        assert!(ctx.globals.syscom.pending_proc.is_none());
        assert!(!ctx.input.vk_down_up_stock(0x02));

        ctx.globals.syscom.menu_open = true;
        ctx.on_gamepad_action(GamepadAction::Cancel, true, false);
        assert!(ctx.input.vk_down_up_stock(0x02));
        ctx.input.clear_all();
        ctx.globals.syscom.menu_open = false;

        ctx.wait.wait_global_movie(true, false);
        ctx.on_gamepad_action(GamepadAction::Cancel, true, false);
        assert!(ctx.input.vk_down_up_stock(0x02));
        ctx.input.clear_all();
        ctx.wait.clear();

        // EXCALL menu should still receive right-click even if hide_mwnd was set.
        ctx.excall_state.ex_call_flag = true;
        ctx.globals.syscom.hide_mwnd.onoff = true;
        ctx.on_gamepad_action(GamepadAction::Cancel, true, false);
        assert!(ctx.input.vk_down_up_stock(0x02));
        assert!(ctx.globals.syscom.hide_mwnd.onoff);
        ctx.input.clear_all();

        // Focused editbox inside EXCALL must not get a synthetic right-click.
        ctx.globals.focused_editbox = Some((1, 0));
        ctx.on_gamepad_action(GamepadAction::Cancel, true, false);
        assert!(!ctx.input.vk_down_up_stock(0x02));
    }
}
