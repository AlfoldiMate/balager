//! Mock data (Hungarian) and date helpers.
//! "Today" is anchored to 2026-05-31 to match the design prototype.

pub const HU_DAYS_SHORT: [&str; 7] = ["H", "K", "Sze", "Cs", "P", "Szo", "V"];
pub const HU_MONTHS: [&str; 12] = [
    "január", "február", "március", "április", "május", "június",
    "július", "augusztus", "szeptember", "október", "november", "december",
];

/// (year, month 1-based, day)
pub const TODAY: (i32, u32, u32) = (2026, 5, 31);

// ---- Civil date math (Howard Hinnant's algorithms) ----

pub fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = (if m <= 2 { y - 1 } else { y }) as i64;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as i64;
    let mp = if m > 2 { m - 3 } else { m + 9 } as i64;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    ((y + if m <= 2 { 1 } else { 0 }) as i32, m, d)
}

pub fn days_in_month(y: i32, m: u32) -> u32 {
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    (days_from_civil(ny, nm, 1) - days_from_civil(y, m, 1)) as u32
}

/// Monday-based weekday: 0 = Monday … 6 = Sunday.
pub fn weekday(y: i32, m: u32, d: u32) -> u32 {
    (days_from_civil(y, m, d) + 3).rem_euclid(7) as u32
}

pub fn iso_week(y: i32, m: u32, d: u32) -> u32 {
    let dn = days_from_civil(y, m, d);
    let dow = (dn + 3).rem_euclid(7); // 0 = Monday
    let thursday = dn + (3 - dow);
    let (ty, _, _) = civil_from_days(thursday);
    let jan1 = days_from_civil(ty, 1, 1);
    ((thursday - jan1) / 7 + 1) as u32
}

pub fn iso(y: i32, m: u32, d: u32) -> String {
    format!("{y:04}-{m:02}-{d:02}")
}

pub fn parse_iso(s: &str) -> (i32, u32, u32) {
    let y = s[0..4].parse().unwrap_or(2026);
    let m = s[5..7].parse().unwrap_or(1);
    let d = s[8..10].parse().unwrap_or(1);
    (y, m, d)
}

/// "június 5." style date.
pub fn hu_date(s: &str) -> String {
    let (_, m, d) = parse_iso(s);
    format!("{} {}.", HU_MONTHS[(m - 1) as usize], d)
}

/// "június 5–7." within one month, otherwise "május 29. – június 1.".
pub fn hu_range(from: &str, to: &str) -> String {
    let (_, fm, fd) = parse_iso(from);
    let (_, tm, td) = parse_iso(to);
    if fm == tm {
        format!("{} {}–{}.", HU_MONTHS[(fm - 1) as usize], fd, td)
    } else {
        format!("{} – {}", hu_date(from), hu_date(to))
    }
}

pub fn day_count(from: &str, to: &str) -> i64 {
    let (fy, fm, fd) = parse_iso(from);
    let (ty, tm, td) = parse_iso(to);
    days_from_civil(ty, tm, td) - days_from_civil(fy, fm, fd) + 1
}

// ---- Users ----

#[derive(PartialEq)]
pub struct User {
    pub id: &'static str,
    pub name: &'static str,
    pub color: &'static str,
    pub approver: bool,
}

pub static USERS: &[User] = &[
    User { id: "anna", name: "Anna", color: "#3f8aa3", approver: true },
    User { id: "bela", name: "Béla", color: "#c47b4a", approver: true },
    User { id: "csaba", name: "Csaba", color: "#5a9b7c", approver: false },
    User { id: "dora", name: "Dóra", color: "#a86fa0", approver: false },
    User { id: "eszter", name: "Eszter", color: "#cf9d3a", approver: false },
    User { id: "gabor", name: "Gábor", color: "#6b86b3", approver: false },
];

pub const ME: &str = "csaba";

pub fn user(id: &str) -> &'static User {
    USERS.iter().find(|u| u.id == id).unwrap_or(&USERS[0])
}

pub fn initials(name: &str) -> String {
    name.chars().take(2).collect()
}

// ---- Reservations ----
// status: "pending" | "reject" | "closed" | "open"  (closed/open are the two
// access modes of an approved reservation)

#[derive(PartialEq)]
pub struct Reservation {
    pub id: &'static str,
    pub title: &'static str,
    pub from: &'static str,
    pub to: &'static str,
    pub status: &'static str,
    pub owner: &'static str,
    pub attendees: &'static [&'static str],
    pub approvals: &'static [(&'static str, &'static str)],
    pub note: &'static str,
    pub reject_reason: &'static str,
}

const RES: Reservation = Reservation {
    id: "", title: "", from: "", to: "", status: "pending", owner: "",
    attendees: &[], approvals: &[], note: "", reject_reason: "",
};

pub static RESERVATIONS: &[Reservation] = &[
    Reservation {
        id: "r1", title: "Anyáék hétvégéje", from: "2026-05-22", to: "2026-05-24",
        status: "closed", owner: "anna", attendees: &["anna", "bela"],
        approvals: &[("anna", "approved"), ("bela", "approved")], ..RES
    },
    Reservation {
        id: "r2", title: "Pünkösdi hosszú hétvége", from: "2026-05-29", to: "2026-06-01",
        status: "open", owner: "bela", attendees: &["bela", "csaba", "eszter", "gabor"],
        approvals: &[("anna", "approved"), ("bela", "approved")],
        note: "Mindenki jöhet! Grillezés szombaton.", ..RES
    },
    Reservation {
        id: "r3", title: "Csaba — barátokkal", from: "2026-06-05", to: "2026-06-07",
        status: "pending", owner: "csaba", attendees: &["csaba"],
        approvals: &[("anna", "approved"), ("bela", "pending")], ..RES
    },
    Reservation {
        id: "r4", title: "Dóra szülinapja", from: "2026-06-12", to: "2026-06-14",
        status: "reject", owner: "dora", attendees: &["dora"],
        approvals: &[("anna", "rejected"), ("bela", "pending")],
        reject_reason: "Ezen a hétvégén festők dolgoznak a házban, sajnos nem fér bele.", ..RES
    },
    Reservation {
        id: "r5", title: "Nagy családi munkahétvége", from: "2026-06-19", to: "2026-06-21",
        status: "open", owner: "anna", attendees: &["anna", "bela", "csaba", "dora"],
        approvals: &[("anna", "approved"), ("bela", "approved")],
        note: "Kerti munkák + stégjavítás. Aki tud, jöjjön segíteni.", ..RES
    },
    Reservation {
        id: "r6", title: "Eszter & Gábor", from: "2026-06-26", to: "2026-06-28",
        status: "closed", owner: "eszter", attendees: &["eszter", "gabor"],
        approvals: &[("anna", "approved"), ("bela", "approved")], ..RES
    },
    Reservation {
        id: "r7", title: "Béla — horgászás", from: "2026-07-03", to: "2026-07-05",
        status: "pending", owner: "bela", attendees: &["bela"],
        approvals: &[("anna", "pending"), ("bela", "approved")], ..RES
    },
    Reservation {
        id: "r8", title: "Nyári nyaralás", from: "2026-07-10", to: "2026-07-19",
        status: "open", owner: "anna", attendees: &["anna", "csaba", "dora", "eszter"],
        approvals: &[("anna", "approved"), ("bela", "approved")], ..RES
    },
];

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

pub fn reservation_on(iso_day: &str) -> Option<&'static Reservation> {
    RESERVATIONS.iter().find(|r| iso_day >= r.from && iso_day <= r.to)
}

pub fn reservation(id: &str) -> Option<&'static Reservation> {
    RESERVATIONS.iter().find(|r| r.id == id)
}

// ---- Tasks ----

#[derive(PartialEq)]
pub struct SubTask {
    pub id: &'static str,
    pub title: &'static str,
    pub done: bool,
}

#[derive(PartialEq)]
pub struct TaskEvent {
    pub label: &'static str,
    pub res_id: &'static str,
}

#[derive(PartialEq)]
pub struct Task {
    pub id: &'static str,
    pub title: &'static str,
    pub done: bool,
    pub due: &'static str,
    pub assignee: &'static str,
    pub recurring: &'static str,
    pub event: Option<TaskEvent>,
    pub subs: &'static [SubTask],
}

const TASK: Task = Task {
    id: "", title: "", done: false, due: "", assignee: ME,
    recurring: "", event: None, subs: &[],
};

#[derive(PartialEq)]
pub struct TaskGroup {
    pub id: &'static str,
    pub name: &'static str,
    pub tasks: &'static [Task],
}

pub static TASK_GROUPS: &[TaskGroup] = &[
    TaskGroup { id: "g1", name: "Stég és csónak", tasks: &[
        Task {
            id: "t1", title: "Stég pallóinak cseréje", due: "2026-06-20", assignee: "bela",
            event: Some(TaskEvent { label: "Munkahétvége", res_id: "r5" }),
            subs: &[
                SubTask { id: "s1", title: "Faanyag beszerzése", done: true },
                SubTask { id: "s2", title: "Régi pallók bontása", done: false },
                SubTask { id: "s3", title: "Új pallók rögzítése", done: false },
            ], ..TASK
        },
        Task { id: "t2", title: "Csónak vízre tétele", done: true, due: "2026-05-20", assignee: "csaba", ..TASK },
        Task { id: "t3", title: "Mentőmellények ellenőrzése", assignee: "anna", ..TASK },
    ]},
    TaskGroup { id: "g2", name: "Kert", tasks: &[
        Task { id: "t4", title: "Fűnyírás", recurring: "Kéthetente", assignee: "gabor", ..TASK },
        Task {
            id: "t5", title: "Nádas tisztítása a partnál", due: "2026-06-21", assignee: "bela",
            event: Some(TaskEvent { label: "Munkahétvége", res_id: "r5" }), ..TASK
        },
        Task { id: "t6", title: "Virágágyások öntözése", recurring: "Hetente", assignee: "eszter", ..TASK },
    ]},
    TaskGroup { id: "g3", name: "Ház", tasks: &[
        Task {
            id: "t7", title: "Nappali kifestése", due: "2026-06-13", assignee: "dora",
            subs: &[
                SubTask { id: "s4", title: "Falak előkészítése", done: true },
                SubTask { id: "s5", title: "Festék vásárlás", done: true },
                SubTask { id: "s6", title: "Festés", done: false },
            ], ..TASK
        },
        Task { id: "t8", title: "Kémény ellenőrzés", recurring: "Évente", assignee: "anna", ..TASK },
        Task { id: "t9", title: "Riasztó elemcsere", done: true, due: "2026-05-15", assignee: "csaba", ..TASK },
    ]},
    TaskGroup { id: "g4", name: "Beszerzés", tasks: &[
        Task { id: "t10", title: "Tűzifa rendelés télre", due: "2026-09-01", assignee: "bela", ..TASK },
        Task {
            id: "t11", title: "Grill szén és kellékek", due: "2026-05-29", assignee: "csaba",
            event: Some(TaskEvent { label: "Pünkösd", res_id: "r2" }), ..TASK
        },
    ]},
];

// ---- Discussions ----
// kind: "general" | "reservation" | "task"

#[derive(PartialEq)]
pub struct PollOpt {
    pub id: &'static str,
    pub label: &'static str,
    pub sub: &'static str,
    pub votes: &'static [&'static str],
}

#[derive(PartialEq)]
pub struct Poll {
    pub question: &'static str,
    /// "date" | "list"
    pub ptype: &'static str,
    /// "single" | "multi"
    pub mode: &'static str,
    pub options: &'static [PollOpt],
}

#[derive(PartialEq)]
pub struct Msg {
    pub id: &'static str,
    pub author: &'static str,
    pub time: &'static str,
    pub text: &'static str,
    pub system: bool,
    pub votes: u32,
    pub down: u32,
    pub voted: i8,
    pub pinned: bool,
    pub image: &'static str,
    pub poll: Option<Poll>,
    pub replies: &'static [Msg],
}

const MSG: Msg = Msg {
    id: "", author: "", time: "", text: "", system: false,
    votes: 0, down: 0, voted: 0, pinned: false, image: "", poll: None, replies: &[],
};

#[derive(PartialEq)]
pub struct Thread {
    pub id: &'static str,
    pub title: &'static str,
    pub kind: &'static str,
    pub link_id: &'static str,
    pub link_label: &'static str,
    pub author: &'static str,
    pub time: &'static str,
    pub closed: bool,
    pub replies: u32,
    pub votes: u32,
    pub excerpt: &'static str,
    pub messages: &'static [Msg],
}

pub static THREADS: &[Thread] = &[
    Thread {
        id: "d1", title: "Pünkösd — ki mit hoz?", kind: "reservation",
        link_id: "r2", link_label: "Pünkösdi hosszú hétvége",
        author: "bela", time: "tegnap", closed: false, replies: 7, votes: 5,
        excerpt: "Csináljunk egy listát, hogy ne háromféle szénsavas üdítő legyen megint.",
        messages: &[
            Msg {
                id: "m1", author: "bela", time: "tegnap 18:20",
                text: "Sziasztok! Pünkösdre csináljunk egy listát, hogy ki mit hoz, nehogy megint három láda ásványvíz legyen és semmi más. Én hozom a húst a grillre.",
                votes: 6, down: 1, voted: 1, pinned: true,
                replies: &[
                    Msg { id: "m1a", author: "eszter", time: "tegnap 19:02", text: "Szuper! Én hozok salátát és desszertet.", votes: 3, ..MSG },
                    Msg { id: "m1b", author: "csaba", time: "tegnap 19:40", text: "Pékáru és reggeli rajtam. Meg a szén — az amúgy is a feladatlistámon van.", votes: 2, ..MSG },
                ], ..MSG
            },
            Msg { id: "sys1", system: true, text: "Gábor csatlakozott a foglaláshoz", time: "ma 08:10", ..MSG },
            Msg {
                id: "p1", author: "anna", time: "ma 08:40",
                poll: Some(Poll {
                    question: "Melyik este legyen a nagy grillezés?", ptype: "date", mode: "single",
                    options: &[
                        PollOpt { id: "o1", label: "Péntek", sub: "máj. 29.", votes: &["anna", "csaba"] },
                        PollOpt { id: "o2", label: "Szombat", sub: "máj. 30.", votes: &["bela", "eszter", "gabor", "dora"] },
                        PollOpt { id: "o3", label: "Vasárnap", sub: "máj. 31.", votes: &[] },
                    ],
                }), ..MSG
            },
            Msg {
                id: "m2", author: "gabor", time: "ma 08:12",
                text: "Sziasztok, én is jövök! Hozok egy láda helyi bort a fonyódi pincészetből.",
                votes: 4, voted: 1, ..MSG
            },
            Msg {
                id: "p2", author: "csaba", time: "ma 10:15",
                poll: Some(Poll {
                    question: "Ki mit vállal a közös bevásárlásból?", ptype: "list", mode: "multi",
                    options: &[
                        PollOpt { id: "l1", label: "Hús és pácok", sub: "", votes: &["bela"] },
                        PollOpt { id: "l2", label: "Zöldség, saláta", sub: "", votes: &["eszter"] },
                        PollOpt { id: "l3", label: "Pékáru, reggeli", sub: "", votes: &["csaba"] },
                        PollOpt { id: "l4", label: "Italok", sub: "", votes: &["gabor", "anna"] },
                        PollOpt { id: "l5", label: "Jég és szén", sub: "", votes: &[] },
                    ],
                }), ..MSG
            },
            Msg {
                id: "m3", author: "anna", time: "ma 09:30",
                text: "Tökéletes. Akkor italt már ne hozzon más. Pokrócokat és napozóágyakat én előkészítem.",
                votes: 2, ..MSG
            },
        ],
    },
    Thread {
        id: "d2", title: "Stég pallócseréje — milyen fát vegyünk?", kind: "task",
        link_id: "t1", link_label: "Stég pallóinak cseréje",
        author: "bela", time: "2 napja", closed: false, replies: 4, votes: 8,
        excerpt: "Vörösfenyő vagy borovi? A borovi olcsóbb, de a vörösfenyő bírja a vizet.",
        messages: &[
            Msg {
                id: "m4", author: "bela", time: "2 napja",
                text: "A pallócseréhez milyen fát vegyünk? Vörösfenyő drágább, de jobban bírja a vizet. Borovi olcsóbb. Vélemények?",
                votes: 5, down: 3,
                replies: &[
                    Msg { id: "m4a", author: "csaba", time: "2 napja", text: "Egyértelműen vörösfenyő. A stég a vízben van, ne spóroljunk rossz helyen.", votes: 6, down: 1, voted: 1, ..MSG },
                ], ..MSG
            },
            Msg {
                id: "m5", author: "anna", time: "tegnap",
                text: "Egyetértek a vörösfenyővel. Megnéztem, a keszthelyi telepen van készleten.",
                votes: 3, pinned: true, ..MSG
            },
        ],
    },
    Thread {
        id: "d3", title: "Új napozóágyak a teraszra", kind: "general",
        link_id: "", link_label: "",
        author: "dora", time: "4 napja", closed: false, replies: 3, votes: 2,
        excerpt: "A régiek már nagyon elhasználódtak. Találtam pár jó ajánlatot.",
        messages: &[
            Msg {
                id: "m6", author: "dora", time: "4 napja",
                text: "A régi napozóágyak már szétesnek. Találtam néhány jó ajánlatot, beraktam egy képet az egyikről.",
                votes: 2, image: "napozóágy — termékfotó", ..MSG
            },
        ],
    },
    Thread {
        id: "d4", title: "Dóra szülinapi hétvégéje", kind: "reservation",
        link_id: "r4", link_label: "Dóra szülinapja",
        author: "dora", time: "5 napja", closed: true, replies: 5, votes: 1,
        excerpt: "Sajnos ütközik a festéssel — kerestünk másik időpontot.",
        messages: &[
            Msg { id: "m7", author: "dora", time: "5 napja", text: "Szeretném a szülinapomat a háznál tartani jún. 12–14.", votes: 1, ..MSG },
            Msg { id: "sys2", system: true, text: "Anna elutasította a foglalást: „festők dolgoznak a házban”", time: "5 napja", ..MSG },
            Msg { id: "m8", author: "anna", time: "4 napja", text: "Nagyon sajnálom Dóri! A festést régóta szervezzük erre a hétvégére. Mit szólnál a következő hétvégéhez?", votes: 2, ..MSG },
            Msg { id: "sys3", system: true, text: "A beszélgetést Anna lezárta", time: "3 napja", ..MSG },
        ],
    },
];

pub static THREAD_FILTERS: &[(&str, &str)] = &[
    ("all", "Mind"),
    ("general", "Általános"),
    ("reservation", "Foglalás"),
    ("task", "Feladat"),
];

// ---- Notifications ----

#[derive(PartialEq)]
pub struct Notif {
    pub id: &'static str,
    pub icon: &'static str,
    pub tone: &'static str,
    pub unread: bool,
    pub time: &'static str,
    pub who: &'static str,
    pub text: &'static str,
}

pub static NOTIFS: &[Notif] = &[
    Notif { id: "n1", icon: "check", tone: "open", unread: true, time: "2 órája",
        who: "bela", text: "jóváhagyta a Pünkösdi hosszú hétvége foglalást." },
    Notif { id: "n2", icon: "users", tone: "closed", unread: true, time: "3 órája",
        who: "gabor", text: "csatlakozott a Pünkösdi hosszú hétvége nyitott foglaláshoz." },
    Notif { id: "n3", icon: "chat", tone: "", unread: true, time: "5 órája",
        who: "csaba", text: "új üzenetet írt: „Stég pallócseréje — milyen fát vegyünk?”" },
    Notif { id: "n4", icon: "clock", tone: "pending", unread: false, time: "tegnap",
        who: "system", text: "Emlékeztető: a Grill szén és kellékek feladat határideje ma." },
    Notif { id: "n5", icon: "x", tone: "reject", unread: false, time: "tegnap",
        who: "anna", text: "elutasította a Dóra szülinapja foglalást." },
    Notif { id: "n6", icon: "flag", tone: "reed", unread: false, time: "2 napja",
        who: "anna", text: "lezárta a Dóra szülinapi hétvégéje beszélgetést." },
];

// ---- Notification settings ----

#[derive(PartialEq)]
pub struct NotifRow {
    pub id: &'static str,
    pub label: &'static str,
    pub sub: &'static str,
    pub email: bool,
    pub push: bool,
}

#[derive(PartialEq)]
pub struct NotifGroup {
    pub id: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    pub rows: &'static [NotifRow],
}

pub static NOTIF_GROUPS: &[NotifGroup] = &[
    NotifGroup { id: "res", label: "Foglalások", icon: "calendar", rows: &[
        NotifRow { id: "res_decision", label: "Foglalásomat jóváhagyták / elutasították", sub: "", email: true, push: true },
        NotifRow { id: "res_request", label: "Új foglalási kérés jóváhagyásra", sub: "Csak engedélyezőknek", email: true, push: true },
        NotifRow { id: "res_join", label: "Valaki csatlakozott a foglalásomhoz", sub: "", email: false, push: true },
    ]},
    NotifGroup { id: "task", label: "Feladatok", icon: "tasks", rows: &[
        NotifRow { id: "task_assigned", label: "Új feladatot rendeltek hozzám", sub: "", email: true, push: true },
        NotifRow { id: "task_due", label: "Közelgő határidő emlékeztető", sub: "", email: false, push: true },
    ]},
    NotifGroup { id: "disc", label: "Beszélgetések", icon: "chat", rows: &[
        NotifRow { id: "disc_reply", label: "Válasz a témáimra", sub: "", email: false, push: true },
        NotifRow { id: "disc_mention", label: "Megemlítettek egy üzenetben", sub: "", email: true, push: true },
        NotifRow { id: "disc_new", label: "Új beszélgetés indult", sub: "", email: false, push: false },
    ]},
];
