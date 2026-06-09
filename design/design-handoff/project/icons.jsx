// icons.jsx — Balager icon set. Stroke-based, 24x24, currentColor.
const ICON_PATHS = {
  calendar: '<rect x="3" y="4.5" width="18" height="16" rx="2.5"/><path d="M3 9h18M8 2.5v4M16 2.5v4"/>',
  tasks: '<path d="M4 6.5h10M4 12h10M4 17.5h6"/><path d="M17.5 5.5l1.6 1.6 3-3.4M17.5 11l1.6 1.6 3-3.4"/>',
  chat: '<path d="M21 11.5a7.5 7.5 0 0 1-10.5 6.9L4 20l1.6-4.2A7.5 7.5 0 1 1 21 11.5Z"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v5.5M12 7.6v.2"/>',
  bell: '<path d="M18 9a6 6 0 1 0-12 0c0 6-2.2 7.5-2.2 7.5h16.4S18 15 18 9Z"/><path d="M13.7 20a2 2 0 0 1-3.4 0"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.2-3.2"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  lock: '<rect x="4.5" y="10.5" width="15" height="10" rx="2.5"/><path d="M8 10.5V7.5a4 4 0 0 1 8 0v3"/>',
  users: '<circle cx="9" cy="8" r="3.4"/><path d="M3 20c0-3.3 2.7-5.5 6-5.5s6 2.2 6 5.5"/><path d="M16 5.2a3.3 3.3 0 0 1 0 6.3M17.5 14.8c2.2.6 3.5 2.4 3.5 5.2"/>',
  check: '<path d="M4.8 12.6 8.8 16.6 17.4 7"/>',
  checkmini: '<path d="M5 12.4 9 16.4 17.4 7"/>',
  pin: '<path d="M15 3.5 20.5 9l-3 1-4 4-.5 4.5L9.5 15 4 20.5M9.5 15 14 10.5"/>',
  arrowup: '<path d="M12 19V6M6 11l6-6 6 6"/>',
  arrowdown: '<path d="M12 5v13M6 13l6 6 6-6"/>',
  reply: '<path d="M9 7 4 12l5 5M4 12h9a7 7 0 0 1 7 7v.5"/>',
  dots: '<circle cx="5" cy="12" r="1.6"/><circle cx="12" cy="12" r="1.6"/><circle cx="19" cy="12" r="1.6"/>',
  x: '<path d="M6 6l12 12M18 6 6 18"/>',
  chevright: '<path d="m9 5 7 7-7 7"/>',
  chevleft: '<path d="m15 5-7 7 7 7"/>',
  chevdown: '<path d="m5 9 7 7 7-7"/>',
  panelopen: '<rect x="3" y="4.5" width="18" height="15" rx="2.5"/><path d="M9 4.5v15"/>',
  repeat: '<path d="M4 9a5 5 0 0 1 5-5h8m0 0-3-3m3 3-3 3"/><path d="M20 15a5 5 0 0 1-5 5H7m0 0 3 3m-3-3 3-3"/>',
  clock: '<circle cx="12" cy="12" r="8.5"/><path d="M12 7.5V12l3 2"/>',
  flag: '<path d="M5 21V4M5 4h11l-2 4 2 4H5"/>',
  link: '<path d="M9.5 14.5 14.5 9.5M10 6.5l1.5-1.5a4 4 0 0 1 5.6 5.6L15.5 12M13.5 17l-1.4 1.4a4 4 0 0 1-5.6-5.6L8 11.4"/>',
  sun: '<circle cx="12" cy="12" r="4.5"/><path d="M12 2v2.5M12 19.5V22M2 12h2.5M19.5 12H22M5 5l1.8 1.8M17.2 17.2 19 19M19 5l-1.8 1.8M6.8 17.2 5 19"/>',
  waves: '<path d="M2 8c2 0 2.5-2 4.5-2S9 8 11 8s2.5-2 4.5-2S18 8 20 8M2 13c2 0 2.5-2 4.5-2S9 13 11 13s2.5-2 4.5-2S18 13 20 13M2 18c2 0 2.5-2 4.5-2S9 18 11 18s2.5-2 4.5-2S18 18 20 18"/>',
  send: '<path d="M4 12 20 4l-4 16-4-7-8-1Z"/>',
  paperclip: '<path d="M20 11.5 12.4 19a5 5 0 0 1-7-7l8-8a3.2 3.2 0 0 1 4.6 4.6l-7.7 7.7a1.5 1.5 0 0 1-2.2-2.1l7-7"/>',
  filter: '<path d="M3 5h18l-7 8v6l-4-2v-4L3 5Z"/>',
  list: '<path d="M8 6h13M8 12h13M8 18h13M3.5 6h.01M3.5 12h.01M3.5 18h.01"/>',
  folder: '<path d="M3 7a2 2 0 0 1 2-2h4l2 2.5h8a2 2 0 0 1 2 2V18a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z"/>',
  settings: '<circle cx="12" cy="12" r="3"/><path d="M12 2.5v2.5M12 19v2.5M21.5 12H19M5 12H2.5M18.5 5.5 16.8 7.2M7.2 16.8 5.5 18.5M18.5 18.5 16.8 16.8M7.2 7.2 5.5 5.5"/>',
  home: '<path d="M4 11 12 4l8 7M6 9.5V20h12V9.5"/>',
  map: '<path d="M9 4 3 6.5v13L9 17l6 2.5 6-2.5v-13L15 6.5 9 4ZM9 4v13M15 6.5v13"/>',
  mail: '<rect x="3" y="5" width="18" height="14" rx="2.5"/><path d="m4 7 8 6 8-6"/>',
  phone: '<rect x="6" y="2.5" width="12" height="19" rx="3"/><path d="M10.5 18.5h3"/>',
  logout: '<path d="M15 5V4a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h7a2 2 0 0 0 2-2v-1M10 12h11m0 0-3-3m3 3-3 3"/>',
  shield: '<path d="M12 3 5 6v5c0 4.5 3 8 7 10 4-2 7-5.5 7-10V6l-7-3Z"/>',
  camera: '<rect x="3" y="6.5" width="18" height="13" rx="2.5"/><circle cx="12" cy="13" r="3.3"/><path d="M8.5 6.5 10 4h4l1.5 2.5"/>',
  // ---- Balaton decorative glyphs ----
  sailboat: '<path d="M12 3 4.5 16H12V3Z"/><path d="M14 8l4.2 8H14"/><path d="M3 18.5h18l-2.2 3.2H5.2L3 18.5Z"/>',
  fish: '<path d="M16 12c0-2.6-3.1-4.8-7-4.8-2.6 0-4.9 1-6.4 2.6 1 .9 1.4 1.6 1.5 2.2-.1.6-.5 1.3-1.5 2.2 1.5 1.6 3.8 2.6 6.4 2.6 3.9 0 7-2.2 7-4.8Z"/><path d="M16 12c1.5-1.6 3.1-2.1 5-2.1-1 1.3-1 2.9 0 4.2-1.9 0-3.5-.5-5-2.1Z"/><circle cx="6.5" cy="11" r="0.7"/>',
  cocktail: '<path d="M5 5h14l-7 7-7-7Z"/><path d="M12 12v6"/><path d="M8.5 20h7"/><path d="m14.5 5.2 2.8-2.4"/><circle cx="18" cy="2.6" r="1.1"/>',
  icecream: '<path d="M8 9.5a4 4 0 1 1 8 0"/><path d="M7.2 9.5h9.6L12 21.2 7.2 9.5Z"/>',
  anchor: '<circle cx="12" cy="4" r="2"/><path d="M12 6v14M5 12H4a8 8 0 0 0 16 0h-1M8.5 11 5.5 12.2M15.5 11l3 1.2"/>',
  umbrella: '<path d="M12 3a9 9 0 0 1 9 8H3a9 9 0 0 1 9-8Z"/><path d="M12 11v8M12 19a2.2 2.2 0 0 0 3.6 1"/>',
  lounger: '<path d="M3 16.6l5.8-1.1 1-4.9 2.1.4-1 4.7 8.3-1.6"/><path d="M6.6 16.1v2.6M18.7 13.6l.5 5"/>',
  beachball: '<circle cx="12" cy="12" r="9"/><path d="M12 3c3.2 3 3.2 15 0 18M3.4 9.2c5.2 2.1 12 2.1 17.2 0M4 15c4.2-1.6 11.8-1.6 16 0"/>',
  palm: '<path d="M12 21V9"/><path d="M12 9c-3-2-6-1.5-8 .5 2.5.2 4 .8 5 2M12 9c3-2 6-1.5 8 .5-2.5.2-4 .8-5 2M12 9c-1-3 .5-6 3-7-.5 2.5-.3 4 .3 5.5M12 9c1-3-.5-6-3-7 .5 2.5.3 4-.3 5.5"/>',
  balaton: '<path d="M2.6 19.2C2 17.3 4 15.7 6.7 14.1 10.2 12 14.2 9.4 17.6 7.4 19.6 6.2 21.2 5.7 21.8 6.2 22.4 6.7 21.9 8.1 20.4 9.7 18.2 12 14.4 14.4 10.8 16.6 8 18.3 5.4 20 4.2 20.7 3.4 21.2 2.9 20.6 2.6 19.2Z"/>',
  collapseall: '<path d="M7 4l5 5 5-5M7 20l5-5 5 5"/>',
  expandall: '<path d="M7 9l5-5 5 5M7 15l5 5 5-5"/>',
};

function Icon({ name, size = 20, stroke = 2, fill = false, style, className }) {
  const d = ICON_PATHS[name] || '';
  return (
    <svg width={size} height={size} viewBox="0 0 24 24"
      fill={fill ? 'currentColor' : 'none'} stroke={fill ? 'none' : 'currentColor'}
      strokeWidth={stroke} strokeLinecap="round" strokeLinejoin="round"
      style={style} className={className}
      dangerouslySetInnerHTML={{ __html: d }} />
  );
}

window.Icon = Icon;
