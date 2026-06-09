//! Info: house rules & how-to.

use dioxus::prelude::*;

use crate::icons::Icon;
use crate::state::AppState;

static TILES: &[(&str, &str, &str)] = &[
    ("calendar", "Foglalások", "Görgethető heti naptár. Jelölj ki szabad napokat és kérj foglalást — két engedélyező hagyja jóvá."),
    ("tasks", "Feladatok", "Csoportosított teendők alfeladatokkal, ismétlődéssel. Egyes feladatok nyitott foglalássá válhatnak."),
    ("chat", "Beszélgetések", "Nyiss témát bármiről. A foglalások és feladatok eseményei automatikusan ide kerülnek."),
    ("bell", "Értesítések", "E-mail és push értesítés a fontos eseményekről. A profilodban szabhatod testre."),
];

static RULES: &[(&str, &str)] = &[
    ("Foglalás = napok.", "Kezdő és záró időpont nincs, csak teljes napok. Egy hétvége jellemzően péntektől vasárnapig tart."),
    ("Két jóváhagyás kell.", "Egy foglalás akkor elfogadott, ha minden engedélyező (Anna és Béla) jóváhagyta. Bárki elutasíthatja, indoklással."),
    ("Zárt vs. nyitott.", "Zárt foglalásnál csak a foglaló kezeli a résztvevőket. Nyitott foglaláshoz bárki csatlakozhat."),
    ("Hagyd tisztán.", "Távozás előtt: szemét kivihető, hűtő kiürítve, redőnyök leengedve, gázcsap elzárva."),
    ("Stég és csónak.", "A mentőmellény kötelező a gyerekeknek. A csónakkulcs a bejárati szekrény felső fiókjában."),
];

#[component]
pub fn InfoTool() -> Element {
    let state = use_context::<AppState>();
    let device_mobile = (state.is_mobile)();
    let img_style = if device_mobile {
        "width: 100%; height: 130px; flex-shrink: 0;"
    } else {
        "width: 220px; height: 130px; flex-shrink: 0;"
    };

    rsx! {
        div { class: "bg-content bg-fade",
            div { class: "nf-wrap",
                div { class: "nf-hero",
                    div { class: "sunblob" }
                    div { style: "position: relative;",
                        div {
                            class: "bg-chip",
                            style: "background: rgba(255,255,255,.18); border: none; color: #fff; margin-bottom: 12px;",
                            Icon { name: "waves", size: 14.0 }
                            " Balaton · Családi nyaraló"
                        }
                        h2 { "Üdv a Balagerben!" }
                        p { "Itt egy helyen kezeljük a nyaraló foglalásait, a közös teendőket és minden beszélgetést. Az alábbiakban a legfontosabb tudnivalók és a házirend." }
                    }
                }

                div { class: "nf-grid",
                    for (icon, h, p) in TILES {
                        div { class: "bg-card nf-tile", key: "{h}",
                            div { class: "ic", Icon { name: "{icon}", size: 20.0 } }
                            h4 { "{h}" }
                            p { "{p}" }
                        }
                    }
                }

                div { class: "bg-card nf-rules",
                    h4 { "Házirend és tudnivalók" }
                    for (i, (lead, t)) in RULES.iter().enumerate() {
                        div { class: "nf-rule", key: "{i}",
                            span { class: "no", {format!("{:02}", i + 1)} }
                            p {
                                span { class: "lead", "{lead}" }
                                " {t}"
                            }
                        }
                    }
                }

                div { class: "bg-card nf-rules", style: "display: flex; gap: 18px; align-items: center; flex-wrap: wrap;",
                    div { class: "ph-img", style: "{img_style}", "nyaraló — fotó" }
                    div { style: "flex: 1; min-width: 180px;",
                        h4 { style: "margin-bottom: 6px;", "Cím és megközelítés" }
                        p { style: "font-size: 14px; color: var(--ink-2);", "8638 Balatonlelle, Nád utca 7." }
                        button { class: "bg-btn ghost sm", style: "margin-top: 12px;",
                            Icon { name: "map", size: 15.0 }
                            " Térkép megnyitása"
                        }
                    }
                }
            }
        }
    }
}
