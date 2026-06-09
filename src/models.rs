//! Shared DTOs (client ⇄ server) and Hungarian date/label helpers.

use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};

pub const HU_DAYS_SHORT: [&str; 7] = ["H", "K", "Sze", "Cs", "P", "Szo", "V"];
pub const HU_MONTHS: [&str; 12] = [
    "január", "február", "március", "április", "május", "június",
    "július", "augusztus", "szeptember", "október", "november", "december",
];

// ---- Dates ----

pub fn today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

pub fn parse_iso(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

pub fn iso(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

/// "június 5." style date.
pub fn hu_date(s: &str) -> String {
    match parse_iso(s) {
        Some(d) => format!("{} {}.", HU_MONTHS[d.month0() as usize], d.day()),
        None => s.to_string(),
    }
}

/// "június 5–7." within one month, otherwise "május 29. – június 1.".
pub fn hu_range(from: &str, to: &str) -> String {
    match (parse_iso(from), parse_iso(to)) {
        (Some(f), Some(t)) if f.month() == t.month() && f.year() == t.year() => {
            format!("{} {}–{}.", HU_MONTHS[f.month0() as usize], f.day(), t.day())
        }
        _ => format!("{} – {}", hu_date(from), hu_date(to)),
    }
}

pub fn day_count(from: &str, to: &str) -> i64 {
    match (parse_iso(from), parse_iso(to)) {
        (Some(f), Some(t)) => (t - f).num_days() + 1,
        _ => 0,
    }
}

/// Relative Hungarian timestamp for an epoch-seconds value:
/// "most", "X perce", "ma 10:15", "tegnap 18:20", "3 napja", "május 9.".
pub fn time_label(epoch: i64) -> String {
    let now = chrono::Local::now();
    let then = match chrono::DateTime::from_timestamp(epoch, 0) {
        Some(t) => t.with_timezone(&chrono::Local),
        None => return String::new(),
    };
    let secs = (now - then).num_seconds().max(0);
    if secs < 60 {
        return "most".into();
    }
    if secs < 3600 {
        return format!("{} perce", secs / 60);
    }
    let today = now.date_naive();
    let day = then.date_naive();
    if day == today {
        return format!("ma {}", then.format("%H:%M"));
    }
    if day == today - Duration::days(1) {
        return format!("tegnap {}", then.format("%H:%M"));
    }
    let days = (today - day).num_days();
    if days < 14 {
        return format!("{days} napja");
    }
    format!("{} {}.", HU_MONTHS[day.month0() as usize], day.day())
}

// ---- Status helpers ----

pub fn status_label(status: &str) -> &'static str {
    match status {
        "pending" => "Függőben",
        "reject" => "Elutasítva",
        "closed" => "Zárt foglalás",
        "open" => "Nyitott",
        _ => "",
    }
}

pub fn status_icon(status: &str) -> &'static str {
    match status {
        "pending" => "clock",
        "reject" => "x",
        "closed" => "lock",
        "open" => "users",
        _ => "info",
    }
}

pub const RECURRING_OPTIONS: [(&str, &str); 4] = [
    ("weekly", "Hetente"),
    ("biweekly", "Kéthetente"),
    ("monthly", "Havonta"),
    ("yearly", "Évente"),
];

pub fn recurring_label(key: &str) -> &'static str {
    RECURRING_OPTIONS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, l)| *l)
        .unwrap_or("")
}

// ---- Notification preference catalogue (labels are UI-side) ----

pub struct PrefRowMeta {
    pub key: &'static str,
    pub label: &'static str,
    pub sub: &'static str,
}

pub struct PrefGroupMeta {
    pub label: &'static str,
    pub icon: &'static str,
    pub rows: &'static [PrefRowMeta],
}

pub static PREF_GROUPS: &[PrefGroupMeta] = &[
    PrefGroupMeta { label: "Foglalások", icon: "calendar", rows: &[
        PrefRowMeta { key: "res_decision", label: "Foglalásomat jóváhagyták / elutasították", sub: "" },
        PrefRowMeta { key: "res_request", label: "Új foglalási kérés jóváhagyásra", sub: "Csak engedélyezőknek" },
        PrefRowMeta { key: "res_join", label: "Valaki csatlakozott a foglalásomhoz", sub: "" },
    ]},
    PrefGroupMeta { label: "Feladatok", icon: "tasks", rows: &[
        PrefRowMeta { key: "task_assigned", label: "Új feladatot rendeltek hozzám", sub: "" },
        PrefRowMeta { key: "task_due", label: "Közelgő határidő emlékeztető", sub: "" },
    ]},
    PrefGroupMeta { label: "Beszélgetések", icon: "chat", rows: &[
        PrefRowMeta { key: "disc_reply", label: "Válasz a témáimra", sub: "" },
        PrefRowMeta { key: "disc_new", label: "Új beszélgetés indult", sub: "" },
    ]},
];

// ---- DTOs ----

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserDto {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub color: String,
    pub approver: bool,
    pub active: bool,
}

impl UserDto {
    pub fn initials(&self) -> String {
        self.name.chars().take(2).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApprovalDto {
    pub user_id: i64,
    /// "pending" | "approved" | "rejected"
    pub status: String,
    pub comment: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReservationDto {
    pub id: i64,
    pub title: String,
    pub from: String,
    pub to: String,
    /// "closed" | "open"
    pub access: String,
    /// Derived: "pending" | "reject" | "closed" | "open"
    pub status: String,
    pub owner: i64,
    pub attendees: Vec<i64>,
    pub approvals: Vec<ApprovalDto>,
    pub note: String,
    pub reject_reason: String,
    pub rejected_by: Option<i64>,
    pub thread_id: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubTaskDto {
    pub id: i64,
    pub title: String,
    pub done: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskEventDto {
    pub res_id: i64,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskDto {
    pub id: i64,
    pub group_id: i64,
    pub title: String,
    pub done: bool,
    pub due: Option<String>,
    pub assignee: Option<i64>,
    /// "weekly" | "biweekly" | "monthly" | "yearly"
    pub recurring: Option<String>,
    pub event: Option<TaskEventDto>,
    pub subs: Vec<SubTaskDto>,
    pub thread_id: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskGroupDto {
    pub id: i64,
    pub name: String,
    pub tasks: Vec<TaskDto>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThreadDto {
    pub id: i64,
    pub title: String,
    /// "general" | "reservation" | "task"
    pub kind: String,
    pub link_id: Option<i64>,
    pub link_label: String,
    pub author: i64,
    pub created_at: i64,
    pub closed: bool,
    pub replies: i64,
    pub votes: i64,
    pub excerpt: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PollOptDto {
    pub id: i64,
    pub label: String,
    pub sub: String,
    pub votes: Vec<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PollDto {
    pub id: i64,
    pub question: String,
    /// "date" | "list"
    pub ptype: String,
    /// "single" | "multi"
    pub mode: String,
    pub options: Vec<PollOptDto>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MessageDto {
    pub id: i64,
    pub author: Option<i64>,
    pub created_at: i64,
    pub body: String,
    pub system: bool,
    pub pinned: bool,
    pub up: i64,
    pub down: i64,
    /// -1 | 0 | 1 for the requesting user
    pub my_vote: i64,
    pub poll: Option<PollDto>,
    pub replies: Vec<MessageDto>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThreadDetailDto {
    pub thread: ThreadDto,
    pub messages: Vec<MessageDto>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotifDto {
    pub id: i64,
    pub icon: String,
    pub tone: String,
    pub unread: bool,
    pub created_at: i64,
    pub text: String,
    pub link_kind: Option<String>,
    pub link_id: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrefDto {
    pub key: String,
    pub email: bool,
    pub push: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewReservation {
    pub title: String,
    pub from: String,
    pub to: String,
    /// "closed" | "open"
    pub access: String,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskInput {
    pub group_id: i64,
    pub title: String,
    pub due: Option<String>,
    pub assignee: Option<i64>,
    pub recurring: Option<String>,
}
