//! Reservations: continuous scrollable week calendar + working detail/new panel.

use chrono::{Datelike, Months, NaiveDate};
use dioxus::prelude::*;

use crate::api;
use crate::common::{Avatar, ErrorNote, StatusChip};
use crate::icons::Icon;
use crate::login::clean_err;
use crate::models::{
    day_count, hu_range, iso, status_icon, today, NewReservation, ReservationDto, HU_DAYS_SHORT,
    HU_MONTHS,
};
use crate::state::AppState;

#[derive(Clone, PartialEq)]
enum Panel {
    New,
    View(i64),
}

#[component]
pub fn ResHeaderLeft() -> Element {
    let state = use_context::<AppState>();
    rsx! {
        button {
            class: "bg-btn ghost",
            title: "Ugrás a mai napra",
            onclick: move |_| {
                let mut j = state.today_jump;
                let v = j();
                j.set(v + 1);
            },
            Icon { name: "calendar", size: 16.0, stroke: 2.2 }
            " Ma"
        }
        span { class: "cal-hint", "Jelölj ki szabad napokat új foglaláshoz" }
    }
}

#[component]
pub fn ResHeaderRight() -> Element {
    rsx! {
        div { class: "cal-legend",
            for (cls, label) in [("pending", "Függőben"), ("reject", "Elutasítva"), ("closed", "Zárt"), ("open", "Nyitott")] {
                span { class: "lg", key: "{cls}",
                    span {
                        class: "sw",
                        style: "background: var(--st-{cls}-bg); border-color: var(--st-{cls}-bd);",
                    }
                    " {label}"
                }
            }
        }
    }
}

struct Month {
    year: i32,
    month: u32,
    weeks: Vec<Vec<Option<(u32, String)>>>,
}

/// Rolling window: previous month → +10 months, weeks aligned to Monday.
fn build_months() -> Vec<Month> {
    let start = today()
        .with_day(1)
        .unwrap()
        .checked_sub_months(Months::new(1))
        .unwrap();
    (0..12)
        .filter_map(|i| start.checked_add_months(Months::new(i)))
        .map(|first| {
            let year = first.year();
            let month = first.month();
            let days = days_in_month(year, month);
            let lead = first.weekday().num_days_from_monday() as usize;
            let mut weeks: Vec<Vec<Option<(u32, String)>>> = Vec::new();
            let mut week: Vec<Option<(u32, String)>> = vec![None; 7];
            let mut col = lead;
            for day in 1..=days {
                let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
                week[col] = Some((day, iso(date)));
                col += 1;
                if col == 7 {
                    weeks.push(std::mem::replace(&mut week, vec![None; 7]));
                    col = 0;
                }
            }
            if col > 0 {
                weeks.push(week);
            }
            Month { year, month, weeks }
        })
        .collect()
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let next = first.checked_add_months(Months::new(1)).unwrap();
    (next - first).num_days() as u32
}

struct Bar {
    res: ReservationDto,
    start: usize,
    end: usize,
    cap_l: bool,
    cap_r: bool,
}

fn week_bars(week: &[Option<(u32, String)>], reservations: &[ReservationDto]) -> Vec<Bar> {
    let mut bars = Vec::new();
    for r in reservations {
        let mut s: Option<usize> = None;
        let mut e = 0usize;
        for (ci, cell) in week.iter().enumerate() {
            if let Some((_, di)) = cell {
                if di.as_str() >= r.from.as_str() && di.as_str() <= r.to.as_str() {
                    if s.is_none() {
                        s = Some(ci);
                    }
                    e = ci;
                }
            }
        }
        if let Some(s) = s {
            let cap_l = week[s].as_ref().map(|(_, di)| *di == r.from).unwrap_or(false);
            let cap_r = week[e].as_ref().map(|(_, di)| *di == r.to).unwrap_or(false);
            bars.push(Bar { res: r.clone(), start: s, end: e, cap_l, cap_r });
        }
    }
    bars
}

fn reservation_on(iso_day: &str, reservations: &[ReservationDto]) -> Option<ReservationDto> {
    reservations
        .iter()
        .find(|r| iso_day >= r.from.as_str() && iso_day <= r.to.as_str())
        .cloned()
}

#[component]
pub fn ReservationsTool() -> Element {
    let state = use_context::<AppState>();
    let device_mobile = (state.is_mobile)();
    let reservations = state.reservations_list();

    let mut sel = use_signal(|| None::<(String, String)>);
    let mut panel = use_signal(|| None::<Panel>);

    // Open the panel when another tool deep-links to a reservation.
    use_effect(move || {
        if let Some((id, _n)) = (state.focus_res)() {
            sel.set(None);
            panel.set(Some(Panel::View(id)));
        }
    });

    // Scroll to today on mount and whenever the "Ma" button is pressed.
    use_effect(move || {
        let _ = (state.today_jump)();
        document::eval(
            r#"
            requestAnimationFrame(() => {
                const c = document.getElementById('cal-scroll');
                const t = document.getElementById('cal-today');
                if (c && t) c.scrollTop += t.getBoundingClientRect().top - c.getBoundingClientRect().top - 96;
            });
            "#,
        );
    });

    let months = use_hook(|| std::rc::Rc::new(build_months()));
    let today_iso = iso(today());

    let mut click_day = move |di: String| {
        // Selection is for requesting: skip days already taken by an active
        // reservation.
        if reservation_on(&di, &state.reservations_list())
            .map(|r| r.status != "reject")
            .unwrap_or(false)
        {
            return;
        }
        let current = sel();
        match current {
            None => sel.set(Some((di.clone(), di))),
            Some((s, e)) => {
                if di < s {
                    sel.set(Some((di, e)));
                } else if di > e {
                    sel.set(Some((s, di)));
                } else {
                    sel.set(Some((di.clone(), di)));
                }
            }
        }
        if !matches!(panel(), Some(Panel::New) | None) {
            panel.set(None);
        }
    };

    let selection = sel();
    let sel_days = selection
        .as_ref()
        .map(|(s, e)| day_count(s, e))
        .unwrap_or(0);

    rsx! {
        div { class: "bg-content bg-fade", id: "cal-scroll",
            div { class: "cal-wrap",
                if device_mobile {
                    div { class: "tool-toolbar",
                        div { class: "tt-left", ResHeaderLeft {} }
                        div { class: "tt-right", ResHeaderRight {} }
                    }
                }

                div { class: "cal-grid",
                    div { class: "cal-head",
                        div { class: "wk" }
                        for (i, d) in HU_DAYS_SHORT.iter().enumerate() {
                            div { key: "{d}", class: if i >= 5 { "dh we" } else { "dh" }, "{d}" }
                        }
                    }
                    for mo in months.iter() {
                        div { class: "cal-month-label", key: "ml{mo.year}-{mo.month}",
                            "{HU_MONTHS[(mo.month - 1) as usize]} {mo.year}"
                        }
                        for (wi, week) in mo.weeks.iter().enumerate() {
                            div { class: "cal-week", key: "w{mo.year}-{mo.month}-{wi}",
                                div { class: "cal-wknum",
                                    span { "hét" }
                                    b {
                                        if let Some((d, _)) = week.iter().flatten().next() {
                                            "{NaiveDate::from_ymd_opt(mo.year, mo.month, *d).unwrap().iso_week().week()}"
                                        }
                                    }
                                }
                                for (ci, cell) in week.iter().enumerate() {
                                    if let Some((day, di)) = cell {
                                        DayCell {
                                            key: "{di}",
                                            day: *day,
                                            di: di.clone(),
                                            today: *di == today_iso,
                                            status: reservation_on(di, &reservations).map(|r| r.status).unwrap_or_default(),
                                            sel_mid: selection.as_ref().map(|(s, e)| di >= s && di <= e).unwrap_or(false),
                                            sel_edge: selection.as_ref().map(|(s, e)| di == s || di == e).unwrap_or(false),
                                            onpick: move |d: String| click_day(d),
                                        }
                                    } else {
                                        div { key: "ph{mo.month}-{wi}-{ci}", class: "cal-day ph" }
                                    }
                                }
                                {
                                    let bars = week_bars(week, &reservations);
                                    rsx! {
                                        if !bars.is_empty() {
                                            div { class: "cal-week-bars",
                                                for (bi, b) in bars.into_iter().enumerate() {
                                                    {
                                                        let owner = state.user(b.res.owner);
                                                        let res_id = b.res.id;
                                                        let mut cls = format!("cal-bar {}", b.res.status);
                                                        if b.cap_l { cls.push_str(" cap-l"); }
                                                        if b.cap_r { cls.push_str(" cap-r"); }
                                                        rsx! {
                                                            div {
                                                                key: "b{bi}",
                                                                class: "{cls}",
                                                                style: "grid-column: {b.start + 1} / {b.end + 2};",
                                                                title: "{b.res.title} · {owner.name}",
                                                                onclick: move |e| {
                                                                    e.stop_propagation();
                                                                    sel.set(None);
                                                                    panel.set(Some(Panel::View(res_id)));
                                                                },
                                                                if b.cap_l {
                                                                    Icon { name: "{status_icon(&b.res.status)}", size: 12.0, stroke: 2.4, style: "flex-shrink: 0;" }
                                                                    span { class: "bt", "{b.res.title}" }
                                                                    span { class: "cal-owner",
                                                                        span { class: "cal-oav", style: "background: {owner.color};", "{owner.initials()}" }
                                                                        "{owner.name}"
                                                                    }
                                                                } else {
                                                                    span { class: "bt cont", "{b.res.title}" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some((s, e)) = selection.clone() {
                    div { class: "cal-selbar",
                        Icon { name: "calendar", size: 18.0 }
                        div {
                            div { style: "font-weight: 700; font-size: 14px;", "{hu_range(&s, &e)}" }
                            div { style: "font-size: 12px; color: rgba(255,255,255,.65);", "{sel_days} nap kiválasztva" }
                        }
                        button {
                            class: "bg-btn sun sm",
                            style: "margin-left: 8px;",
                            onclick: move |_| panel.set(Some(Panel::New)),
                            Icon { name: "check", size: 15.0 }
                            " Foglalás kérése"
                        }
                        button { class: "x", onclick: move |_| sel.set(None),
                            Icon { name: "x", size: 17.0 }
                        }
                    }
                }
            }
        }

        match panel() {
            Some(Panel::New) => rsx! {
                NewReservationPanel {
                    selection: selection.clone(),
                    onclose: move |created: bool| {
                        panel.set(None);
                        sel.set(None);
                        if created {
                            let mut r = state.reservations;
                            r.restart();
                        }
                    },
                }
            },
            Some(Panel::View(res_id)) => rsx! {
                ViewReservationPanel {
                    res_id,
                    onclose: move |_| panel.set(None),
                }
            },
            None => rsx! {},
        }
    }
}

#[component]
fn DayCell(
    day: u32,
    di: String,
    today: bool,
    status: String,
    sel_mid: bool,
    sel_edge: bool,
    onpick: EventHandler<String>,
) -> Element {
    let mut cls = String::from("cal-day");
    if today {
        cls.push_str(" today");
    }
    if !status.is_empty() {
        cls.push_str(" has ");
        cls.push_str(&status);
    }
    if sel_mid {
        cls.push_str(" sel-mid");
    }
    if sel_edge {
        cls.push_str(" sel");
    }
    let di2 = di.clone();
    rsx! {
        div {
            class: "{cls}",
            id: if today { "cal-today" },
            onclick: move |_| onpick.call(di2.clone()),
            div { class: "dn", span { class: "num", "{day}" } }
        }
    }
}

#[component]
fn NewReservationPanel(selection: Option<(String, String)>, onclose: EventHandler<bool>) -> Element {
    let state = use_context::<AppState>();
    let device_mobile = (state.is_mobile)();
    let mut title = use_signal(|| "Hétvége a háznál".to_string());
    let mut access = use_signal(|| "closed".to_string());
    let mut note = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut busy = use_signal(|| false);

    let (range, days, from, to) = match &selection {
        Some((s, e)) => (hu_range(s, e), day_count(s, e), s.clone(), e.clone()),
        None => (String::new(), 0, String::new(), String::new()),
    };

    let approvers: Vec<_> = state.users_list().into_iter().filter(|u| u.approver).collect();
    let acc = access();

    let submit = move |_| {
        if busy() || from.is_empty() {
            return;
        }
        busy.set(true);
        error.set(String::new());
        let input = NewReservation {
            title: title(),
            from: from.clone(),
            to: to.clone(),
            access: access(),
            note: note(),
        };
        spawn(async move {
            match api::reservations::create_reservation(input).await {
                Ok(_) => onclose.call(true),
                Err(e) => error.set(clean_err(&e.to_string())),
            }
            busy.set(false);
        });
    };

    rsx! {
        aside { class: "bg-panel",
            if device_mobile {
                div { class: "sheet-grab" }
            }
            div { class: "ph",
                div { style: "flex: 1;",
                    h3 { "Új foglalás" }
                    div { style: "font-size: 13px; color: var(--ink-3); margin-top: 3px;", "{range} · {days} nap" }
                }
                button { class: "bg-iconbtn", onclick: move |_| onclose.call(false),
                    Icon { name: "x", size: 18.0 }
                }
            }
            div { class: "pbody",
                ErrorNote { message: error() }
                div { class: "bg-field",
                    label { "Foglalás neve" }
                    input {
                        class: "bg-input",
                        value: "{title}",
                        oninput: move |e| title.set(e.value()),
                    }
                }
                div { class: "bg-field",
                    label { "Hozzáférés" }
                    div { class: "bg-seg",
                        button {
                            class: if acc == "closed" { "on closed" } else { "" },
                            onclick: move |_| access.set("closed".into()),
                            Icon { name: "lock", size: 15.0, stroke: 2.2 }
                            " Zárt"
                        }
                        button {
                            class: if acc == "open" { "on open" } else { "" },
                            onclick: move |_| access.set("open".into()),
                            Icon { name: "users", size: 15.0, stroke: 2.2 }
                            " Nyitott"
                        }
                    }
                    p { style: "font-size: 12.5px; color: var(--ink-3); margin-top: 8px;",
                        if acc == "closed" {
                            "Csak te kezelheted a résztvevőket."
                        } else {
                            "Bármely családtag csatlakozhat ehhez a hétvégéhez."
                        }
                    }
                }
                div { class: "bg-field",
                    label { "Üzenet az engedélyezőknek" }
                    textarea {
                        class: "bg-input",
                        rows: 3,
                        placeholder: "Pl. nyugis hétvége a tónál…",
                        style: "resize: none;",
                        value: "{note}",
                        oninput: move |e| note.set(e.value()),
                    }
                }
                div { class: "bg-field",
                    label { "Engedélyezés szükséges" }
                    div { class: "appr",
                        for u in approvers {
                            div { class: "appr-row", key: "{u.id}",
                                Avatar { user: u.clone(), size: "sm" }
                                span { class: "nm", "{u.name}" }
                                span { class: "bg-chip", style: "margin-left: auto;",
                                    Icon { name: "clock", size: 13.0 }
                                    " Vár"
                                }
                            }
                        }
                    }
                }
            }
            div { class: "pfoot",
                button { class: "bg-btn ghost", onclick: move |_| onclose.call(false), "Mégse" }
                button {
                    class: "bg-btn",
                    style: "flex: 1; justify-content: center;",
                    disabled: busy(),
                    onclick: submit,
                    Icon { name: "check", size: 17.0 }
                    " Foglalás kérése"
                }
            }
        }
    }
}

#[component]
fn ViewReservationPanel(res_id: i64, onclose: EventHandler<()>) -> Element {
    let state = use_context::<AppState>();
    let device_mobile = (state.is_mobile)();
    let mut error = use_signal(String::new);
    let mut reject_open = use_signal(|| false);
    let mut reject_comment = use_signal(String::new);
    let mut menu = use_signal(|| false);
    let mut add_open = use_signal(|| false);

    let Some(res) = state.reservations_list().into_iter().find(|r| r.id == res_id) else {
        return rsx! {};
    };
    let me = state.me();
    let is_owner = res.owner == me.id;
    let is_open = res.access == "open";
    let owner = state.user(res.owner);

    let run = move |fut:
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ServerFnError>>>>| {
        let mut state = state;
        spawn(async move {
            match fut.await {
                Ok(()) => {
                    state.reservations.restart();
                    state.threads.restart();
                    error.set(String::new());
                }
                Err(e) => error.set(clean_err(&e.to_string())),
            }
        });
    };

    let my_pending = me.approver
        && res
            .approvals
            .iter()
            .any(|a| a.user_id == me.id && a.status == "pending");
    let non_attendees: Vec<_> = state
        .users_list()
        .into_iter()
        .filter(|u| !res.attendees.contains(&u.id))
        .collect();

    rsx! {
        aside { class: "bg-panel",
            if device_mobile {
                div { class: "sheet-grab" }
            }
            div { class: "ph",
                div { style: "flex: 1;",
                    div { style: "margin-bottom: 7px;", StatusChip { status: "{res.status}" } }
                    h3 { "{res.title}" }
                    div { style: "font-size: 13px; color: var(--ink-3); margin-top: 4px; display: flex; align-items: center; gap: 6px;",
                        Icon { name: "calendar", size: 14.0 }
                        " {hu_range(&res.from, &res.to)}"
                    }
                }
                button { class: "bg-iconbtn", onclick: move |_| onclose.call(()),
                    Icon { name: "x", size: 18.0 }
                }
            }
            div { class: "pbody",
                ErrorNote { message: error() }
                if res.status == "reject" && !res.reject_reason.is_empty() {
                    div { style: "padding: 12px 14px; border-radius: 11px; background: var(--st-reject-bg); border: 1px solid var(--st-reject-bd); color: var(--st-reject-ink); font-size: 13.5px;",
                        b { style: "display: flex; align-items: center; gap: 6px; margin-bottom: 4px;",
                            Icon { name: "x", size: 14.0 }
                            " Elutasítva — {res.rejected_by.map(|id| state.user(id).name).unwrap_or_default()}"
                        }
                        "{res.reject_reason}"
                    }
                }
                if !res.note.is_empty() {
                    div { style: "padding: 12px 14px; border-radius: 11px; background: var(--surface-2); border: 1px solid var(--line); font-size: 13.5px; color: var(--ink-2);",
                        "{res.note}"
                    }
                }

                div { class: "bg-field",
                    label { "Foglaló" }
                    div { style: "display: flex; align-items: center; gap: 10px;",
                        Avatar { user: owner.clone(), size: "sm" }
                        span { style: "font-weight: 600;", "{owner.name}" }
                        if owner.approver {
                            span { class: "bg-chip reed", style: "height: 22px;", "Engedélyező" }
                        }
                    }
                }

                div { class: "bg-field",
                    label { "Hozzáférés" }
                    div { class: "bg-seg",
                        button {
                            class: if !is_open { "on closed" } else { "" },
                            disabled: !is_owner,
                            onclick: move |_| {
                                if is_owner && is_open {
                                    run(Box::pin(api::reservations::set_access(res_id, "closed".into())));
                                }
                            },
                            Icon { name: "lock", size: 15.0, stroke: 2.2 }
                            " Zárt"
                        }
                        button {
                            class: if is_open { "on open" } else { "" },
                            disabled: !is_owner,
                            onclick: move |_| {
                                if is_owner && !is_open {
                                    run(Box::pin(api::reservations::set_access(res_id, "open".into())));
                                }
                            },
                            Icon { name: "users", size: 15.0, stroke: 2.2 }
                            " Nyitott"
                        }
                    }
                    if !is_owner {
                        p { style: "font-size: 12px; color: var(--ink-3); margin-top: 7px;",
                            "A hozzáférést csak a foglaló módosíthatja."
                        }
                    }
                }

                div { class: "bg-field",
                    label { "Résztvevők · {res.attendees.len()}" }
                    div { style: "display: flex; flex-direction: column; gap: 7px;",
                        for id in res.attendees.clone() {
                            {
                                let u = state.user(id);
                                rsx! {
                                    div { key: "{id}", style: "display: flex; align-items: center; gap: 10px;",
                                        Avatar { user: u.clone(), size: "sm" }
                                        span { style: "font-weight: 500; font-size: 14px;", "{u.name}" }
                                        if id == res.owner {
                                            span { style: "font-size: 12px; color: var(--ink-3); margin-left: auto;", "foglaló" }
                                        } else if is_owner {
                                            button {
                                                class: "bg-iconbtn",
                                                style: "width: 28px; height: 28px; margin-left: auto;",
                                                title: "Eltávolítás",
                                                onclick: move |_| run(Box::pin(api::reservations::set_attendee(res_id, id, false))),
                                                Icon { name: "x", size: 14.0 }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if is_owner && !non_attendees.is_empty() {
                        if add_open() {
                            div { style: "display: flex; flex-direction: column; gap: 6px; margin-top: 10px;",
                                for u in non_attendees.clone() {
                                    button {
                                        key: "{u.id}",
                                        class: "appr-row",
                                        style: "width: 100%; cursor: pointer;",
                                        onclick: move |_| {
                                            add_open.set(false);
                                            run(Box::pin(api::reservations::set_attendee(res_id, u.id, true)));
                                        },
                                        Avatar { user: u.clone(), size: "sm" }
                                        span { class: "nm", "{u.name}" }
                                        Icon { name: "plus", size: 15.0, style: "margin-left: auto; color: var(--water);" }
                                    }
                                }
                            }
                        } else {
                            button {
                                class: "bg-btn ghost sm",
                                style: "margin-top: 11px;",
                                onclick: move |_| add_open.set(true),
                                Icon { name: "plus", size: 15.0 }
                                " Résztvevő hozzáadása"
                            }
                        }
                    }
                    if !is_owner && is_open && res.status != "reject" {
                        if res.attendees.contains(&me.id) {
                            button {
                                class: "bg-btn ghost sm",
                                style: "margin-top: 11px;",
                                onclick: move |_| run(Box::pin(api::reservations::set_attendance(res_id, false))),
                                Icon { name: "x", size: 15.0 }
                                " Lemondom a részvételt"
                            }
                        } else {
                            button {
                                class: "bg-btn sun sm",
                                style: "margin-top: 11px;",
                                onclick: move |_| run(Box::pin(api::reservations::set_attendance(res_id, true))),
                                Icon { name: "plus", size: 15.0 }
                                " Csatlakozom"
                            }
                        }
                    }
                }

                div { class: "bg-field",
                    label { "Engedélyezés" }
                    div { class: "appr",
                        for approval in res.approvals.clone() {
                            {
                                let u = state.user(approval.user_id);
                                rsx! {
                                    div { class: "appr-row", key: "{approval.user_id}",
                                        Avatar { user: u.clone(), size: "sm" }
                                        span { class: "nm", "{u.name}" }
                                        span { class: "st",
                                            if approval.status == "approved" {
                                                span { class: "bg-chip open", style: "height: 22px;",
                                                    Icon { name: "check", size: 13.0 }
                                                    " Jóváhagyta"
                                                }
                                            } else if approval.status == "rejected" {
                                                span { class: "bg-chip reject", style: "height: 22px;",
                                                    Icon { name: "x", size: 13.0 }
                                                    " Elutasította"
                                                }
                                            } else {
                                                span { class: "bg-chip", style: "height: 22px;",
                                                    Icon { name: "clock", size: 13.0 }
                                                    " Vár"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if my_pending {
                        if reject_open() {
                            div { style: "display: flex; flex-direction: column; gap: 8px; margin-top: 10px;",
                                textarea {
                                    class: "bg-input",
                                    rows: 2,
                                    placeholder: "Indoklás (kötelező)…",
                                    style: "resize: none;",
                                    value: "{reject_comment}",
                                    oninput: move |e| reject_comment.set(e.value()),
                                }
                                div { style: "display: flex; gap: 8px;",
                                    button {
                                        class: "bg-btn sm",
                                        style: "background: var(--st-reject-ink); flex: 1; justify-content: center;",
                                        onclick: move |_| {
                                            reject_open.set(false);
                                            run(Box::pin(api::reservations::decide_reservation(res_id, false, reject_comment())));
                                        },
                                        Icon { name: "x", size: 14.0 }
                                        " Elutasítás"
                                    }
                                    button { class: "bg-btn ghost sm", onclick: move |_| reject_open.set(false), "Mégse" }
                                }
                            }
                        } else {
                            div { style: "display: flex; gap: 8px; margin-top: 10px;",
                                button {
                                    class: "bg-btn sm",
                                    style: "background: var(--st-open-ink); flex: 1; justify-content: center;",
                                    onclick: move |_| run(Box::pin(api::reservations::decide_reservation(res_id, true, String::new()))),
                                    Icon { name: "check", size: 14.0 }
                                    " Jóváhagyom"
                                }
                                button {
                                    class: "bg-btn ghost sm",
                                    style: "flex: 1; justify-content: center; color: var(--st-reject-ink);",
                                    onclick: move |_| reject_open.set(true),
                                    Icon { name: "x", size: 14.0 }
                                    " Elutasítom"
                                }
                            }
                        }
                    }
                }
            }
            div { class: "pfoot",
                button {
                    class: "bg-btn ghost",
                    style: "flex: 1; justify-content: center;",
                    onclick: move |_| state.go_discuss("reservation", res_id),
                    Icon { name: "chat", size: 16.0 }
                    " Beszélgetés"
                }
                if is_owner || me.approver {
                    div { class: "ds-menuwrap",
                        button {
                            class: "bg-btn ghost",
                            style: "width: 44px; justify-content: center; padding: 0;",
                            onclick: move |_| {
                                let v = menu();
                                menu.set(!v);
                            },
                            Icon { name: "dots", size: 18.0 }
                        }
                        if menu() {
                            div { class: "ds-menu-scrim", onclick: move |_| menu.set(false) }
                            div { class: "ds-menu", style: "bottom: 46px; top: auto;",
                                button {
                                    class: "ds-menu-item danger",
                                    onclick: move |_| {
                                        menu.set(false);
                                        let mut state = state;
                                        spawn(async move {
                                            match api::reservations::delete_reservation(res_id).await {
                                                Ok(()) => {
                                                    state.reservations.restart();
                                                    state.threads.restart();
                                                    onclose.call(());
                                                }
                                                Err(e) => error.set(clean_err(&e.to_string())),
                                            }
                                        });
                                    },
                                    Icon { name: "x", size: 16.0 }
                                    " Foglalás törlése"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
