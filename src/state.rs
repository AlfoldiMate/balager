//! App-wide UI state shared between the shell (top bar, navigation) and tools.

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::data;

/// Task editor panel target.
#[derive(Clone, PartialEq)]
pub enum TaskPanel {
    New,
    Edit(&'static data::Task),
}

#[derive(Clone, Copy)]
pub struct AppState {
    /// Active tool id: "foglalasok" | "feladatok" | "beszelgetesek" | "informacio" | "beallitasok".
    pub active: Signal<&'static str>,
    /// Tool to return to when leaving settings.
    pub back_to: Signal<&'static str>,
    pub sidebar_open: Signal<bool>,
    pub notif_open: Signal<bool>,
    pub is_mobile: Signal<bool>,
    /// Reservation to focus in the calendar; the counter forces re-focus.
    pub focus_res: Signal<Option<(&'static str, u64)>>,
    /// Incremented by the "Ma" button to re-scroll the calendar to today.
    pub today_jump: Signal<u64>,
    /// Tasks tool: "all" | "active" | "done".
    pub task_filter: Signal<&'static str>,
    pub task_open: Signal<HashMap<&'static str, bool>>,
    pub task_panel: Signal<Option<TaskPanel>>,
    /// Discussions tool: "all" | "general" | "reservation" | "task".
    pub disc_filter: Signal<&'static str>,
    pub active_thread: Signal<Option<&'static str>>,
}

pub fn use_app_state() -> AppState {
    let state = AppState {
        active: use_signal(|| "foglalasok"),
        back_to: use_signal(|| "foglalasok"),
        sidebar_open: use_signal(|| true),
        notif_open: use_signal(|| false),
        is_mobile: use_signal(|| false),
        focus_res: use_signal(|| None),
        today_jump: use_signal(|| 0),
        task_filter: use_signal(|| "all"),
        task_open: use_signal(|| {
            data::TASK_GROUPS.iter().map(|g| (g.id, true)).collect()
        }),
        task_panel: use_signal(|| None),
        disc_filter: use_signal(|| "all"),
        active_thread: use_signal(|| None),
    };
    use_context_provider(|| state)
}

impl AppState {
    pub fn set_active_tool(mut self, id: &'static str) {
        if id != "beszelgetesek" {
            self.active_thread.set(None);
        }
        self.notif_open.set(false);
        self.active.set(id);
    }

    pub fn open_settings(mut self) {
        let current = (self.active)();
        if current != "beallitasok" {
            self.back_to.set(current);
        }
        self.notif_open.set(false);
        self.active.set("beallitasok");
    }

    /// Jump to the discussion thread linked to a reservation or task.
    pub fn go_discuss(mut self, link_id: &str) {
        let thread = data::THREADS
            .iter()
            .find(|t| t.link_id == link_id)
            .unwrap_or(&data::THREADS[0]);
        self.active_thread.set(Some(thread.id));
        self.notif_open.set(false);
        self.active.set("beszelgetesek");
    }

    /// Open a reservation in the calendar (from a task event or thread link).
    pub fn open_event(mut self, res_id: &'static str) {
        let n = (self.focus_res)().map(|(_, n)| n).unwrap_or(0) + 1;
        self.notif_open.set(false);
        self.focus_res.set(Some((res_id, n)));
        self.active.set("foglalasok");
    }

    /// Follow a thread's link chip to its reservation or task.
    pub fn open_link(mut self, kind: &str, id: &'static str) {
        if kind == "reservation" {
            self.open_event(id);
        } else if kind == "task" {
            self.active_thread.set(None);
            self.notif_open.set(false);
            self.active.set("feladatok");
        }
    }
}
