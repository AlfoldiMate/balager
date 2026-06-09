//! Reservations: continuous scrollable week calendar + detail/new panel.

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::common::{Avatar, StatusChip};
use crate::data::{
    self, day_count, days_in_month, hu_range, initials, iso, iso_week, reservation_on,
    status_icon, user, weekday, Reservation, HU_DAYS_SHORT, HU_MONTHS, ME, RESERVATIONS, TODAY,
};
use crate::icons::Icon;
use crate::state::AppState;

#[derive(Clone, PartialEq)]
enum Panel {
    New,
    View(&'static Reservation),
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
        span { class: "cal-hint", "Húzz végig napokon át új foglaláshoz" }
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

/// May → September 2026, weeks aligned to Monday, with out-of-month blanks.
fn build_months() -> Vec<Month> {
    let year = 2026;
    (5u32..=9)
        .map(|m| {
            let days = days_in_month(year, m);
            let lead = weekday(year, m, 1) as usize;
            let mut weeks: Vec<Vec<Option<(u32, String)>>> = Vec::new();
            let mut week: Vec<Option<(u32, String)>> = vec![None; 7];
            let mut col = lead;
            for day in 1..=days {
                week[col] = Some((day, iso(year, m, day)));
                col += 1;
                if col == 7 {
                    weeks.push(std::mem::replace(&mut week, vec![None; 7]));
                    col = 0;
                }
            }
            if col > 0 {
                weeks.push(week);
            }
            Month { year, month: m, weeks }
        })
        .collect()
}

struct Bar {
    res: &'static Reservation,
    start: usize,
    end: usize,
    cap_l: bool,
    cap_r: bool,
}

fn week_bars(week: &[Option<(u32, String)>]) -> Vec<Bar> {
    let mut bars = Vec::new();
    for r in RESERVATIONS {
        let mut s: Option<usize> = None;
        let mut e = 0usize;
        for (ci, cell) in week.iter().enumerate() {
            if let Some((_, di)) = cell {
                if di.as_str() >= r.from && di.as_str() <= r.to {
                    if s.is_none() {
                        s = Some(ci);
                    }
                    e = ci;
                }
            }
        }
        if let Some(s) = s {
            let cap_l = week[s].as_ref().map(|(_, di)| di == r.from).unwrap_or(false);
            let cap_r = week[e].as_ref().map(|(_, di)| di == r.to).unwrap_or(false);
            bars.push(Bar { res: r, start: s, end: e, cap_l, cap_r });
        }
    }
    bars
}

#[component]
pub fn ReservationsTool() -> Element {
    let state = use_context::<AppState>();
    let device_mobile = (state.is_mobile)();

    let mut sel = use_signal(|| None::<(String, String)>);
    let mut panel = use_signal(|| None::<Panel>);
    let mut access_override = use_signal(HashMap::<&'static str, &'static str>::new);

    // Open the panel when another tool deep-links to a reservation.
    use_effect(move || {
        if let Some((id, _n)) = (state.focus_res)() {
            if let Some(r) = data::reservation(id) {
                sel.set(None);
                panel.set(Some(Panel::View(r)));
            }
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
    let today_iso = iso(TODAY.0, TODAY.1, TODAY.2);

    let status_of = move |r: &Reservation| -> &'static str {
        access_override.read().get(r.id).copied().unwrap_or(r.status)
    };

    let mut click_day = move |di: String| {
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
        // Keep the 'new' form open while picking dates on the calendar.
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
                        div { class: "cal-month-label", key: "ml{mo.month}", "{HU_MONTHS[(mo.month - 1) as usize]} {mo.year}" }
                        for (wi, week) in mo.weeks.iter().enumerate() {
                            div { class: "cal-week", key: "w{mo.month}-{wi}",
                                div { class: "cal-wknum",
                                    span { "hét" }
                                    b {
                                        if let Some((d, _)) = week.iter().flatten().next() {
                                            "{iso_week(mo.year, mo.month, *d)}"
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
                                            status: reservation_on(di).map(status_of).unwrap_or("").to_string(),
                                            sel_mid: selection.as_ref().map(|(s, e)| di >= s && di <= e).unwrap_or(false),
                                            sel_edge: selection.as_ref().map(|(s, e)| di == s || di == e).unwrap_or(false),
                                            onpick: move |d: String| click_day(d),
                                        }
                                    } else {
                                        div { key: "ph{mo.month}-{wi}-{ci}", class: "cal-day ph" }
                                    }
                                }
                                {
                                    let bars = week_bars(week);
                                    rsx! {
                                        if !bars.is_empty() {
                                            div { class: "cal-week-bars",
                                                for (bi, b) in bars.iter().enumerate() {
                                                    {
                                                        let st = status_of(b.res);
                                                        let owner = user(b.res.owner);
                                                        let res = b.res;
                                                        let mut cls = format!("cal-bar {st}");
                                                        if b.cap_l { cls.push_str(" cap-l"); }
                                                        if b.cap_r { cls.push_str(" cap-r"); }
                                                        let cap_l = b.cap_l;
                                                        rsx! {
                                                            div {
                                                                key: "b{bi}",
                                                                class: "{cls}",
                                                                style: "grid-column: {b.start + 1} / {b.end + 2};",
                                                                title: "{res.title} · {owner.name}",
                                                                onclick: move |e| {
                                                                    e.stop_propagation();
                                                                    sel.set(None);
                                                                    panel.set(Some(Panel::View(res)));
                                                                },
                                                                if cap_l {
                                                                    Icon { name: "{status_icon(st)}", size: 12.0, stroke: 2.4, style: "flex-shrink: 0;" }
                                                                    span { class: "bt", "{res.title}" }
                                                                    span { class: "cal-owner",
                                                                        span { class: "cal-oav", style: "background: {owner.color};", "{initials(owner.name)}" }
                                                                        "{owner.name}"
                                                                    }
                                                                } else {
                                                                    span { class: "bt cont", "{res.title}" }
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

        if let Some(p) = panel() {
            ReservationPanel {
                panel: p.clone(),
                selection: selection.clone(),
                status: match &p {
                    Panel::View(r) => status_of(r).to_string(),
                    Panel::New => String::new(),
                },
                onclose: move |_| {
                    let was_new = matches!(panel(), Some(Panel::New));
                    panel.set(None);
                    if was_new {
                        sel.set(None);
                    }
                },
                onaccess: move |(id, acc): (&'static str, &'static str)| {
                    access_override.write().insert(id, acc);
                },
            }
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
fn ReservationPanel(
    panel: Panel,
    selection: Option<(String, String)>,
    status: String,
    onclose: EventHandler<()>,
    onaccess: EventHandler<(&'static str, &'static str)>,
) -> Element {
    let state = use_context::<AppState>();
    let device_mobile = (state.is_mobile)();
    let mut new_access = use_signal(|| "closed");

    match panel {
        Panel::New => {
            let (range, days) = selection
                .as_ref()
                .map(|(s, e)| (hu_range(s, e), day_count(s, e)))
                .unwrap_or((String::new(), 0));
            let access = new_access();
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
                        button { class: "bg-iconbtn", onclick: move |_| onclose.call(()),
                            Icon { name: "x", size: 18.0 }
                        }
                    }
                    div { class: "pbody",
                        div { class: "bg-field",
                            label { "Foglalás neve" }
                            input { class: "bg-input", initial_value: "Hétvége a háznál" }
                        }
                        div { class: "bg-field",
                            label { "Hozzáférés" }
                            div { class: "bg-seg",
                                button {
                                    class: if access == "closed" { "on closed" } else { "" },
                                    onclick: move |_| new_access.set("closed"),
                                    Icon { name: "lock", size: 15.0, stroke: 2.2 }
                                    " Zárt"
                                }
                                button {
                                    class: if access == "open" { "on open" } else { "" },
                                    onclick: move |_| new_access.set("open"),
                                    Icon { name: "users", size: 15.0, stroke: 2.2 }
                                    " Nyitott"
                                }
                            }
                            p { style: "font-size: 12.5px; color: var(--ink-3); margin-top: 8px;",
                                if access == "closed" {
                                    "Csak te kezelheted a résztvevőket."
                                } else {
                                    "Bármely családtag csatlakozhat ehhez a hétvégéhez."
                                }
                            }
                        }
                        div { class: "bg-field",
                            label { "Üzenet az engedélyezőknek" }
                            textarea { class: "bg-input", rows: 3, placeholder: "Pl. nyugis hétvége a tónál…", style: "resize: none;" }
                        }
                        div { class: "bg-field",
                            label { "Engedélyezés szükséges" }
                            div { class: "appr",
                                for id in ["anna", "bela"] {
                                    div { class: "appr-row", key: "{id}",
                                        Avatar { id: "{id}", size: "sm" }
                                        span { class: "nm", "{user(id).name}" }
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
                        button { class: "bg-btn ghost", onclick: move |_| onclose.call(()), "Mégse" }
                        button {
                            class: "bg-btn",
                            style: "flex: 1; justify-content: center;",
                            onclick: move |_| onclose.call(()),
                            Icon { name: "check", size: 17.0 }
                            " Foglalás kérése"
                        }
                    }
                }
            }
        }
        Panel::View(res) => {
            let is_owner = res.owner == ME;
            let is_open = status == "open";
            let owner = user(res.owner);
            rsx! {
                aside { class: "bg-panel",
                    if device_mobile {
                        div { class: "sheet-grab" }
                    }
                    div { class: "ph",
                        div { style: "flex: 1;",
                            div { style: "margin-bottom: 7px;", StatusChip { status: "{status}" } }
                            h3 { "{res.title}" }
                            div { style: "font-size: 13px; color: var(--ink-3); margin-top: 4px; display: flex; align-items: center; gap: 6px;",
                                Icon { name: "calendar", size: 14.0 }
                                " {hu_range(res.from, res.to)}"
                            }
                        }
                        button { class: "bg-iconbtn", onclick: move |_| onclose.call(()),
                            Icon { name: "x", size: 18.0 }
                        }
                    }
                    div { class: "pbody",
                        if status == "reject" && !res.reject_reason.is_empty() {
                            div { style: "padding: 12px 14px; border-radius: 11px; background: var(--st-reject-bg); border: 1px solid var(--st-reject-bd); color: var(--st-reject-ink); font-size: 13.5px;",
                                b { style: "display: flex; align-items: center; gap: 6px; margin-bottom: 4px;",
                                    Icon { name: "x", size: 14.0 }
                                    " Elutasítva — Anna"
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
                                Avatar { id: "{res.owner}", size: "sm" }
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
                                        if is_owner {
                                            onaccess.call((res.id, "closed"));
                                        }
                                    },
                                    Icon { name: "lock", size: 15.0, stroke: 2.2 }
                                    " Zárt"
                                }
                                button {
                                    class: if is_open { "on open" } else { "" },
                                    disabled: !is_owner,
                                    onclick: move |_| {
                                        if is_owner {
                                            onaccess.call((res.id, "open"));
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
                                for id in res.attendees {
                                    div { key: "{id}", style: "display: flex; align-items: center; gap: 10px;",
                                        Avatar { id: "{id}", size: "sm" }
                                        span { style: "font-weight: 500; font-size: 14px;", "{user(id).name}" }
                                        if *id == res.owner {
                                            span { style: "font-size: 12px; color: var(--ink-3); margin-left: auto;", "foglaló" }
                                        }
                                    }
                                }
                            }
                            if is_open && res.owner != ME && !res.attendees.contains(&ME) {
                                button { class: "bg-btn sun sm", style: "margin-top: 11px;",
                                    Icon { name: "plus", size: 15.0 }
                                    " Csatlakozom"
                                }
                            }
                        }

                        div { class: "bg-field",
                            label { "Engedélyezés" }
                            div { class: "appr",
                                for (id, st) in res.approvals {
                                    div { class: "appr-row", key: "{id}",
                                        Avatar { id: "{id}", size: "sm" }
                                        span { class: "nm", "{user(id).name}" }
                                        span { class: "st",
                                            if *st == "approved" {
                                                span { class: "bg-chip open", style: "height: 22px;",
                                                    Icon { name: "check", size: 13.0 }
                                                    " Jóváhagyta"
                                                }
                                            } else if *st == "rejected" {
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
                    div { class: "pfoot",
                        button {
                            class: "bg-btn ghost",
                            style: "flex: 1; justify-content: center;",
                            onclick: move |_| state.go_discuss(res.id),
                            Icon { name: "chat", size: 16.0 }
                            " Beszélgetés"
                        }
                        if is_owner {
                            button { class: "bg-btn ghost", style: "width: 44px; justify-content: center; padding: 0;",
                                Icon { name: "dots", size: 18.0 }
                            }
                        }
                    }
                }
            }
        }
    }
}
