//! App-wide UI state and live data resources shared by the shell and tools.

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::api;
use crate::models::{NotifDto, ReservationDto, TaskGroupDto, ThreadDto, UserDto};

/// Task editor panel target.
#[derive(Clone, PartialEq)]
pub enum TaskPanel {
    New,
    Edit(i64),
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
    pub focus_res: Signal<Option<(i64, u64)>>,
    /// Incremented by the "Ma" button to re-scroll the calendar to today.
    pub today_jump: Signal<u64>,
    /// Tasks tool: "all" | "active" | "done".
    pub task_filter: Signal<&'static str>,
    pub task_open: Signal<HashMap<i64, bool>>,
    pub task_panel: Signal<Option<TaskPanel>>,
    /// Discussions tool: "all" | "general" | "reservation" | "task".
    pub disc_filter: Signal<&'static str>,
    pub active_thread: Signal<Option<i64>>,
    pub disc_creating: Signal<bool>,

    /// The logged-in user.
    pub me: Signal<UserDto>,
    pub users: Resource<Vec<UserDto>>,
    pub reservations: Resource<Vec<ReservationDto>>,
    pub groups: Resource<Vec<TaskGroupDto>>,
    pub threads: Resource<Vec<ThreadDto>>,
    pub notifs: Resource<Vec<NotifDto>>,
}

pub fn use_app_state(user: UserDto) -> AppState {
    let me = use_signal(|| user);
    let state = AppState {
        active: use_signal(|| "foglalasok"),
        back_to: use_signal(|| "foglalasok"),
        sidebar_open: use_signal(|| true),
        notif_open: use_signal(|| false),
        is_mobile: use_signal(|| false),
        focus_res: use_signal(|| None),
        today_jump: use_signal(|| 0),
        task_filter: use_signal(|| "all"),
        task_open: use_signal(HashMap::new),
        task_panel: use_signal(|| None),
        disc_filter: use_signal(|| "all"),
        active_thread: use_signal(|| None),
        disc_creating: use_signal(|| false),
        me,
        users: use_resource(|| async {
            api::users::list_users().await.unwrap_or_default()
        }),
        reservations: use_resource(|| async {
            api::reservations::list_reservations().await.unwrap_or_default()
        }),
        groups: use_resource(|| async {
            api::tasks::list_task_groups().await.unwrap_or_default()
        }),
        threads: use_resource(|| async {
            api::discussions::list_threads().await.unwrap_or_default()
        }),
        notifs: use_resource(|| async {
            api::notifications::list_notifications().await.unwrap_or_default()
        }),
    };
    use_context_provider(|| state)
}

impl AppState {
    pub fn me(&self) -> UserDto {
        (self.me)()
    }

    pub fn users_list(&self) -> Vec<UserDto> {
        self.users.value()().unwrap_or_default()
    }

    /// Resolve a user for display; falls back to a placeholder so the UI
    /// stays stable while the users resource loads.
    pub fn user(&self, id: i64) -> UserDto {
        let me = self.me();
        if me.id == id {
            return me;
        }
        self.users_list()
            .into_iter()
            .find(|u| u.id == id)
            .unwrap_or(UserDto {
                id,
                name: "–".into(),
                email: String::new(),
                color: "#879c96".into(),
                approver: false,
                active: false,
            })
    }

    pub fn reservations_list(&self) -> Vec<ReservationDto> {
        self.reservations.value()().unwrap_or_default()
    }

    pub fn groups_list(&self) -> Vec<TaskGroupDto> {
        self.groups.value()().unwrap_or_default()
    }

    pub fn threads_list(&self) -> Vec<ThreadDto> {
        self.threads.value()().unwrap_or_default()
    }

    pub fn notifs_list(&self) -> Vec<NotifDto> {
        self.notifs.value()().unwrap_or_default()
    }

    pub fn set_active_tool(mut self, id: &'static str) {
        if id != "beszelgetesek" {
            self.active_thread.set(None);
            self.disc_creating.set(false);
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

    /// Open (creating on first use) the discussion linked to a reservation or task.
    pub fn go_discuss(self, kind: &'static str, link_id: i64) {
        let mut state = self;
        spawn(async move {
            if let Ok(thread_id) =
                api::discussions::open_or_create_thread(kind.to_string(), link_id).await
            {
                state.threads.restart();
                state.active_thread.set(Some(thread_id));
                state.disc_creating.set(false);
                state.notif_open.set(false);
                state.active.set("beszelgetesek");
            }
        });
    }

    /// Open a reservation in the calendar (from a task event or thread link).
    pub fn open_event(mut self, res_id: i64) {
        let n = (self.focus_res)().map(|(_, n)| n).unwrap_or(0) + 1;
        self.notif_open.set(false);
        self.focus_res.set(Some((res_id, n)));
        self.active.set("foglalasok");
    }

    /// Follow a link chip (thread header) or a notification to its target.
    pub fn open_link(mut self, kind: &str, id: i64) {
        match kind {
            "reservation" => self.open_event(id),
            "task" => {
                self.active_thread.set(None);
                self.notif_open.set(false);
                self.active.set("feladatok");
            }
            "thread" => {
                self.active_thread.set(Some(id));
                self.disc_creating.set(false);
                self.notif_open.set(false);
                self.active.set("beszelgetesek");
            }
            _ => {}
        }
    }
}
