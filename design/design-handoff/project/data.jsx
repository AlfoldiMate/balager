// data.jsx — Balager mock data (Hungarian) + date helpers
// "Today" anchored to 2026-05-31 to match the project clock.

const HU_DAYS_SHORT = ['H', 'K', 'Sze', 'Cs', 'P', 'Szo', 'V'];
const HU_DAYS_FULL  = ['Hétfő', 'Kedd', 'Szerda', 'Csütörtök', 'Péntek', 'Szombat', 'Vasárnap'];
const HU_MONTHS = ['január', 'február', 'március', 'április', 'május', 'június',
  'július', 'augusztus', 'szeptember', 'október', 'november', 'december'];

const TODAY = new Date(2026, 4, 31); // 2026-05-31

function iso(d) {
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${d.getFullYear()}-${m}-${day}`;
}
function addDays(d, n) { const x = new Date(d); x.setDate(x.getDate() + n); return x; }
function mondayOf(d) { const x = new Date(d); const wd = (x.getDay() + 6) % 7; return addDays(x, -wd); }
function huDate(d) { return `${HU_MONTHS[d.getMonth()]} ${d.getDate()}.`; }
function huRange(a, b) {
  const da = new Date(a + 'T00:00'), db = new Date(b + 'T00:00');
  if (da.getMonth() === db.getMonth())
    return `${HU_MONTHS[da.getMonth()]} ${da.getDate()}–${db.getDate()}.`;
  return `${huDate(da)} – ${huDate(db)}`;
}
function isoWeek(d) {
  const x = new Date(Date.UTC(d.getFullYear(), d.getMonth(), d.getDate()));
  const day = (x.getUTCDay() + 6) % 7;
  x.setUTCDate(x.getUTCDate() - day + 3);
  const first = new Date(Date.UTC(x.getUTCFullYear(), 0, 4));
  return 1 + Math.round(((x - first) / 86400000 - 3 + ((first.getUTCDay() + 6) % 7)) / 7);
}

// ---- Users ----
const USERS = {
  anna:   { id: 'anna',   name: 'Anna',   color: '#3f8aa3', role: 'approver' },
  bela:   { id: 'bela',   name: 'Béla',   color: '#c47b4a', role: 'approver' },
  csaba:  { id: 'csaba',  name: 'Csaba',  color: '#5a9b7c', role: 'normal' },
  dora:   { id: 'dora',   name: 'Dóra',   color: '#a86fa0', role: 'normal' },
  eszter: { id: 'eszter', name: 'Eszter', color: '#cf9d3a', role: 'normal' },
  gabor:  { id: 'gabor',  name: 'Gábor',  color: '#6b86b3', role: 'normal' },
};
const ME = 'csaba';
function initials(name) { return name.slice(0, 2); }

// ---- Reservations ----  status: pending | reject | closed | open  · access: closed | open
const RESERVATIONS = [
  { id: 'r1', title: 'Anyáék hétvégéje', from: '2026-05-22', to: '2026-05-24',
    status: 'closed', owner: 'anna', attendees: ['anna', 'bela'],
    approvals: { anna: 'approved', bela: 'approved' } },
  { id: 'r2', title: 'Pünkösdi hosszú hétvége', from: '2026-05-29', to: '2026-06-01',
    status: 'open', owner: 'bela', attendees: ['bela', 'csaba', 'eszter', 'gabor'],
    approvals: { anna: 'approved', bela: 'approved' },
    note: 'Mindenki jöhet! Grillezés szombaton.' },
  { id: 'r3', title: 'Csaba — barátokkal', from: '2026-06-05', to: '2026-06-07',
    status: 'pending', owner: 'csaba', attendees: ['csaba'],
    approvals: { anna: 'approved', bela: 'pending' } },
  { id: 'r4', title: 'Dóra szülinapja', from: '2026-06-12', to: '2026-06-14',
    status: 'reject', owner: 'dora', attendees: ['dora'],
    approvals: { anna: 'rejected', bela: 'pending' },
    rejectReason: 'Ezen a hétvégén festők dolgoznak a házban, sajnos nem fér bele.' },
  { id: 'r5', title: 'Nagy családi munkahétvége', from: '2026-06-19', to: '2026-06-21',
    status: 'open', owner: 'anna', attendees: ['anna', 'bela', 'csaba', 'dora'],
    approvals: { anna: 'approved', bela: 'approved' },
    note: 'Kerti munkák + stégjavítás. Aki tud, jöjjön segíteni.' },
  { id: 'r6', title: 'Eszter & Gábor', from: '2026-06-26', to: '2026-06-28',
    status: 'closed', owner: 'eszter', attendees: ['eszter', 'gabor'],
    approvals: { anna: 'approved', bela: 'approved' } },
  { id: 'r7', title: 'Béla — horgászás', from: '2026-07-03', to: '2026-07-05',
    status: 'pending', owner: 'bela', attendees: ['bela'],
    approvals: { anna: 'pending', bela: 'approved' } },
  { id: 'r8', title: 'Nyári nyaralás', from: '2026-07-10', to: '2026-07-19',
    status: 'open', owner: 'anna', attendees: ['anna', 'csaba', 'dora', 'eszter'],
    approvals: { anna: 'approved', bela: 'approved' } },
];

const STATUS_META = {
  pending: { label: 'Függőben', cls: 'pending' },
  reject:  { label: 'Elutasítva', cls: 'reject' },
  closed:  { label: 'Zárt foglalás', cls: 'closed' },
  open:    { label: 'Nyitott', cls: 'open' },
};

// map date(iso) -> {res, isStart, isEnd}
function reservationOn(isoStr) {
  for (const r of RESERVATIONS) {
    if (isoStr >= r.from && isoStr <= r.to)
      return { res: r, isStart: isoStr === r.from, isEnd: isoStr === r.to };
  }
  return null;
}

// ---- Tasks ----
const TASK_GROUPS = [
  { id: 'g1', name: 'Stég és csónak', tasks: [
    { id: 't1', title: 'Stég pallóinak cseréje', done: false, due: '2026-06-20', assignee: 'bela',
      event: { label: 'Munkahétvége', resId: 'r5' },
      subs: [
        { id: 's1', title: 'Faanyag beszerzése', done: true },
        { id: 's2', title: 'Régi pallók bontása', done: false },
        { id: 's3', title: 'Új pallók rögzítése', done: false },
      ] },
    { id: 't2', title: 'Csónak vízre tétele', done: true, due: '2026-05-20', assignee: 'csaba', subs: [] },
    { id: 't3', title: 'Mentőmellények ellenőrzése', done: false, due: null, assignee: 'anna', subs: [] },
  ]},
  { id: 'g2', name: 'Kert', tasks: [
    { id: 't4', title: 'Fűnyírás', done: false, recurring: 'Kéthetente', assignee: 'gabor',
      subs: [] },
    { id: 't5', title: 'Nádas tisztítása a partnál', done: false, due: '2026-06-21', assignee: 'bela',
      event: { label: 'Munkahétvége', resId: 'r5' }, subs: [] },
    { id: 't6', title: 'Virágágyások öntözése', done: false, recurring: 'Hetente', assignee: 'eszter', subs: [] },
  ]},
  { id: 'g3', name: 'Ház', tasks: [
    { id: 't7', title: 'Nappali kifestése', done: false, due: '2026-06-13', assignee: 'dora', subs: [
        { id: 's4', title: 'Falak előkészítése', done: true },
        { id: 's5', title: 'Festék vásárlás', done: true },
        { id: 's6', title: 'Festés', done: false },
      ] },
    { id: 't8', title: 'Kémény ellenőrzés', done: false, recurring: 'Évente', assignee: 'anna', subs: [] },
    { id: 't9', title: 'Riasztó elemcsere', done: true, due: '2026-05-15', assignee: 'csaba', subs: [] },
  ]},
  { id: 'g4', name: 'Beszerzés', tasks: [
    { id: 't10', title: 'Tűzifa rendelés télre', done: false, due: '2026-09-01', assignee: 'bela', subs: [] },
    { id: 't11', title: 'Grill szén és kellékek', done: false, due: '2026-05-29', assignee: 'csaba',
      event: { label: 'Pünkösd', resId: 'r2' }, subs: [] },
  ]},
];

// ---- Discussions ----  kind: general | reservation | task
const THREADS = [
  { id: 'd1', title: 'Pünkösd — ki mit hoz?', kind: 'reservation', linkId: 'r2', linkLabel: 'Pünkösdi hosszú hétvége',
    author: 'bela', time: 'tegnap', closed: false, replies: 7, votes: 5,
    excerpt: 'Csináljunk egy listát, hogy ne háromféle szénsavas üdítő legyen megint.',
    messages: [
      { id: 'm1', author: 'bela', time: 'tegnap 18:20', text: 'Sziasztok! Pünkösdre csináljunk egy listát, hogy ki mit hoz, nehogy megint három láda ásványvíz legyen és semmi más. Én hozom a húst a grillre.', votes: 6, down: 1, voted: 1, pinned: true, replies: [
        { id: 'm1a', author: 'eszter', time: 'tegnap 19:02', text: 'Szuper! Én hozok salátát és desszertet.', votes: 3, voted: 0 },
        { id: 'm1b', author: 'csaba', time: 'tegnap 19:40', text: 'Pékáru és reggeli rajtam. Meg a szén — az amúgy is a feladatlistámon van.', votes: 2, voted: 0 },
      ]},
      { id: 'sys1', system: true, text: 'Gábor csatlakozott a foglaláshoz', time: 'ma 08:10' },
      { id: 'p1', author: 'anna', time: 'ma 08:40', pinned: false, poll: {
        question: 'Melyik este legyen a nagy grillezés?', type: 'date', mode: 'single',
        options: [
          { id: 'o1', label: 'Péntek', sub: 'máj. 29.', votes: ['anna', 'csaba'] },
          { id: 'o2', label: 'Szombat', sub: 'máj. 30.', votes: ['bela', 'eszter', 'gabor', 'dora'] },
          { id: 'o3', label: 'Vasárnap', sub: 'máj. 31.', votes: [] },
        ] } },
      { id: 'm2', author: 'gabor', time: 'ma 08:12', text: 'Sziasztok, én is jövök! Hozok egy láda helyi bort a fonyódi pincészetből.', votes: 4, voted: 1, pinned: false, replies: [] },
      { id: 'p2', author: 'csaba', time: 'ma 10:15', pinned: false, poll: {
        question: 'Ki mit vállal a közös bevásárlásból?', type: 'list', mode: 'multi',
        options: [
          { id: 'l1', label: 'Hús és pácok', votes: ['bela'] },
          { id: 'l2', label: 'Zöldség, saláta', votes: ['eszter'] },
          { id: 'l3', label: 'Pékáru, reggeli', votes: ['csaba'] },
          { id: 'l4', label: 'Italok', votes: ['gabor', 'anna'] },
          { id: 'l5', label: 'Jég és szén', votes: [] },
        ] } },
      { id: 'm3', author: 'anna', time: 'ma 09:30', text: 'Tökéletes. Akkor italt már ne hozzon más. Pokrócokat és napozóágyakat én előkészítem.', votes: 2, voted: 0, replies: [] },
    ]},
  { id: 'd2', title: 'Stég pallócseréje — milyen fát vegyünk?', kind: 'task', linkId: 't1', linkLabel: 'Stég pallóinak cseréje',
    author: 'bela', time: '2 napja', closed: false, replies: 4, votes: 8,
    excerpt: 'Vörösfenyő vagy borovi? A borovi olcsóbb, de a vörösfenyő bírja a vizet.',
    messages: [
      { id: 'm4', author: 'bela', time: '2 napja', text: 'A pallócseréhez milyen fát vegyünk? Vörösfenyő drágább, de jobban bírja a vizet. Borovi olcsóbb. Vélemények?', votes: 5, down: 3, voted: 0, pinned: false, replies: [
        { id: 'm4a', author: 'csaba', time: '2 napja', text: 'Egyértelműen vörösfenyő. A stég a vízben van, ne spóroljunk rossz helyen.', votes: 6, down: 1, voted: 1 },
      ]},
      { id: 'm5', author: 'anna', time: 'tegnap', text: 'Egyetértek a vörösfenyővel. Megnéztem, a keszthelyi telepen van készleten.', votes: 3, voted: 0, pinned: true, replies: [] },
    ]},
  { id: 'd3', title: 'Új napozóágyak a teraszra', kind: 'general', linkId: null,
    author: 'dora', time: '4 napja', closed: false, replies: 3, votes: 2,
    excerpt: 'A régiek már nagyon elhasználódtak. Találtam pár jó ajánlatot.',
    messages: [
      { id: 'm6', author: 'dora', time: '4 napja', text: 'A régi napozóágyak már szétesnek. Találtam néhány jó ajánlatot, beraktam egy képet az egyikről.', votes: 2, voted: 0, pinned: false, image: true, replies: [] },
    ]},
  { id: 'd4', title: 'Dóra szülinapi hétvégéje', kind: 'reservation', linkId: 'r4', linkLabel: 'Dóra szülinapja',
    author: 'dora', time: '5 napja', closed: true, replies: 5, votes: 1,
    excerpt: 'Sajnos ütközik a festéssel — kerestünk másik időpontot.',
    messages: [
      { id: 'm7', author: 'dora', time: '5 napja', text: 'Szeretném a szülinapomat a háznál tartani jún. 12–14.', votes: 1, voted: 0, pinned: false, replies: [] },
      { id: 'sys2', system: true, text: 'Anna elutasította a foglalást: „festők dolgoznak a házban”', time: '5 napja' },
      { id: 'm8', author: 'anna', time: '4 napja', text: 'Nagyon sajnálom Dóri! A festést régóta szervezzük erre a hétvégére. Mit szólnál a következő hétvégéhez?', votes: 2, voted: 0, pinned: false, replies: [] },
      { id: 'sys3', system: true, text: 'A beszélgetést Anna lezárta', time: '3 napja' },
    ]},
];

const THREAD_FILTERS = [
  { id: 'all', label: 'Mind' },
  { id: 'general', label: 'Általános' },
  { id: 'reservation', label: 'Foglalás' },
  { id: 'task', label: 'Feladat' },
];

// ---- Notifications ----
const NOTIFS = [
  { id: 'n1', icon: 'check', tone: 'open', unread: true, time: '2 órája',
    who: 'bela', text: 'jóváhagyta a Pünkösdi hosszú hétvége foglalást.' },
  { id: 'n2', icon: 'users', tone: 'closed', unread: true, time: '3 órája',
    who: 'gabor', text: 'csatlakozott a Pünkösdi hosszú hétvége nyitott foglaláshoz.' },
  { id: 'n3', icon: 'chat', tone: '', unread: true, time: '5 órája',
    who: 'csaba', text: 'új üzenetet írt: „Stég pallócseréje — milyen fát vegyünk?”' },
  { id: 'n4', icon: 'clock', tone: 'pending', unread: false, time: 'tegnap',
    who: 'system', text: 'Emlékeztető: a Grill szén és kellékek feladat határideje ma.' },
  { id: 'n5', icon: 'x', tone: 'reject', unread: false, time: 'tegnap',
    who: 'anna', text: 'elutasította a Dóra szülinapja foglalást.' },
  { id: 'n6', icon: 'flag', tone: 'reed', unread: false, time: '2 napja',
    who: 'anna', text: 'lezárta a Dóra szülinapi hétvégéje beszélgetést.' },
];

// ---- Notification settings ----
const NOTIF_GROUPS = [
  { id: 'res', label: 'Foglalások', icon: 'calendar', rows: [
    { id: 'res_decision', label: 'Foglalásomat jóváhagyták / elutasították', email: true, push: true },
    { id: 'res_request', label: 'Új foglalási kérés jóváhagyásra', sub: 'Csak engedélyezőknek', email: true, push: true },
    { id: 'res_join', label: 'Valaki csatlakozott a foglalásomhoz', email: false, push: true },
  ]},
  { id: 'task', label: 'Feladatok', icon: 'tasks', rows: [
    { id: 'task_assigned', label: 'Új feladatot rendeltek hozzám', email: true, push: true },
    { id: 'task_due', label: 'Közelgő határidő emlékeztető', email: false, push: true },
  ]},
  { id: 'disc', label: 'Beszélgetések', icon: 'chat', rows: [
    { id: 'disc_reply', label: 'Válasz a témáimra', email: false, push: true },
    { id: 'disc_mention', label: 'Megemlítettek egy üzenetben', email: true, push: true },
    { id: 'disc_new', label: 'Új beszélgetés indult', email: false, push: false },
  ]},
];

Object.assign(window, {
  HU_DAYS_SHORT, HU_DAYS_FULL, HU_MONTHS, TODAY,
  iso, addDays, mondayOf, huDate, huRange, isoWeek,
  USERS, ME, initials, RESERVATIONS, STATUS_META, reservationOn,
  TASK_GROUPS, THREADS, THREAD_FILTERS, NOTIFS, NOTIF_GROUPS,
});
