// shell.jsx — Balager app shell: sidebar, topbar, mobile nav, routing
const { useState: useStateS, useEffect: useEffectS } = React;

const TOOLS = [
{ id: 'foglalasok', label: 'Foglalások', icon: 'calendar', badge: 2 },
{ id: 'feladatok', label: 'Feladatok', icon: 'tasks', badge: 9 },
{ id: 'beszelgetesek', label: 'Beszélgetések', icon: 'chat', badge: 3 },
{ id: 'informacio', label: 'Információ', icon: 'info' }];

const TOOL_SUB = {
  foglalasok: 'Heti naptár · Balatonlelle',
  feladatok: 'Közös teendők',
  beszelgetesek: 'Családi témák',
  informacio: 'Tudnivalók és házirend',
  beallitasok: 'Profil és értesítések'
};
const META = { ...Object.fromEntries(TOOLS.map((t) => [t.id, t.label])), beallitasok: 'Beállítások' };

function Sidebar({ open, setOpen, active, setActive, onSettings }) {
  return (
    <nav className={'bg-side' + (open ? ' open' : '')}>
      <WaterWaves height={230} />
      <div className="brand" data-comment-anchor="1c66210a5c-div-23-7">
        {open ?
        <b className="brand-wordmark">Balager</b> :
        <div className="brand-mini"><BalatonMark width={42} /></div>}
      </div>
      <div className="bg-nav">
        {TOOLS.map((t) =>
        <button key={t.id} className={'bg-navitem' + (active === t.id ? ' active' : '')} onClick={() => setActive(t.id)} title={t.label}>
            <Icon name={t.icon} size={21} stroke={2} />
            {open && <span className="lbl">{t.label}</span>}
            {open && t.badge && <span className="badge-dot">{t.badge}</span>}
            {!open && t.badge && <span className="nav-dot" />}
          </button>
        )}
      </div>
      <div className="side-foot">
        <button className="side-toggle" onClick={() => setOpen((o) => !o)} title="Menü">
          <Icon name="panelopen" size={20} stroke={2} />
          {open && <span className="lbl">Menü összecsukása</span>}
        </button>
        <button className={'side-user' + (active === 'beallitasok' ? ' active' : '')} onClick={onSettings} title="Beállítások" data-comment-anchor="c940a18e28-button-42-9">
          <Avatar id={ME} size="sm" ring={false} />
          {open &&
          <div style={{ minWidth: 0, textAlign: 'left' }}>
              <div style={{ fontWeight: 700, fontSize: 13.5, lineHeight: 1.1 }}>{USERS[ME].name}</div>
              <div style={{ fontSize: 11, color: 'rgba(255,255,255,.65)' }}>Családtag</div>
            </div>
          }
        </button>
      </div>
    </nav>);

}

function Balager({ device, sidebarDefault }) {
  const [active, setActive] = useStateS('foglalasok');
  const [backTo, setBackTo] = useStateS('foglalasok');
  const [open, setOpen] = useStateS(sidebarDefault);
  const [deepThread, setDeepThread] = useStateS(null);
  const [notif, setNotif] = useStateS(false);
  const [focusRes, setFocusRes] = useStateS(null);
  const [leftSlot, setLeftSlot] = useStateS(null);
  const [rightSlot, setRightSlot] = useStateS(null);
  useEffectS(() => {setOpen(sidebarDefault);}, [sidebarDefault]);

  function goDiscuss(item) {
    const th = THREADS.find((t) => t.linkId === item.id);
    setDeepThread(th ? th.id : THREADS[0].id);
    setActive('beszelgetesek');
  }
  function openEvent(resId) {setNotif(false);setFocusRes({ id: resId, n: Date.now() });setActive('foglalasok');}
  function openLink(kind, id) {if (kind === 'reservation') openEvent(id);else if (kind === 'task') {setDeepThread(null);setNotif(false);setActive('feladatok');}}
  function setActiveTool(id) {if (id !== 'beszelgetesek') setDeepThread(null);setNotif(false);setActive(id);}
  function openSettings() {setBackTo(active === 'beallitasok' ? backTo : active);setNotif(false);setActive('beallitasok');}

  const tool = active === 'foglalasok' ?
  <ReservationsTool device={device} onGoDiscuss={goDiscuss} focusRes={focusRes} /> :
  active === 'feladatok' ?
  <TasksTool device={device} onGoDiscuss={goDiscuss} onOpenEvent={openEvent} /> :
  active === 'beszelgetesek' ?
  <DiscussionsTool device={device} initialThreadId={deepThread} onOpenLink={openLink} key={deepThread || 'd'} /> :
  active === 'informacio' ?
  <InfoTool device={device} /> :
  <SettingsView device={device} onBack={() => setActiveTool(backTo)} />;

  if (device === 'mobile') {
    return (
      <div className="bg-app is-mobile">
        <div className="bg-mtop">
          <WaterWaves height={70} />
          <div style={{ flex: 1, minWidth: 0 }}>
            <h1>{META[active]}</h1>
            <div className="sub">{TOOL_SUB[active]}</div>
          </div>
          <button className="mbtn" onClick={() => setNotif(true)}><Icon name="bell" size={19} /><span className="dot" /></button>
          <button className="mbtn" onClick={openSettings} style={{ padding: 0, overflow: 'hidden' }}><Avatar id={ME} size="" ring={false} /></button>
        </div>
        <div className="bg-body">{tool}</div>
        <div className="bg-mnav">
          {TOOLS.map((t) =>
          <button key={t.id} className={active === t.id ? 'on' : ''} onClick={() => setActiveTool(t.id)}>
              <span className="mi"><Icon name={t.icon} size={21} stroke={2} /></span>
              {t.label}
            </button>
          )}
        </div>
        {notif && <NotificationsPopover onClose={() => setNotif(false)} />}
      </div>);

  }

  return (
    <div className="bg-app is-desktop">
      <Sidebar open={open} setOpen={setOpen} active={active} setActive={setActiveTool} onSettings={openSettings} />
      <div className="bg-main">
        <div className="bg-top" data-comment-anchor="f199a31523-div-117-9">
          {active === 'beallitasok' ?
          <button className="bg-btn ghost bg-top-back" onClick={() => setActiveTool(backTo)}><Icon name="chevleft" size={16} stroke={2.4} /> Vissza</button> :
          active === 'informacio' ?
          <div className="title-wrap"><h1>{META[active]}</h1><div className="sub">{TOOL_SUB[active]}</div></div> :
          null}
          <div className="bg-top-actions l" ref={setLeftSlot}></div>
          <div className="bg-top-spacer"></div>
          <div className="bg-top-actions r" ref={setRightSlot}></div>
          <button className={'bg-iconbtn' + (notif ? ' on' : '')} onClick={() => setNotif((n) => !n)} title="Értesítések"><Icon name="bell" size={19} /><span className="dot" /></button>
        </div>
        <div className="bg-body">
          <HeaderCtx.Provider value={{ left: leftSlot, right: rightSlot, device }}>
            {tool}
          </HeaderCtx.Provider>
        </div>
      </div>
      {notif && <NotificationsPopover onClose={() => setNotif(false)} />}
    </div>);

}

window.Balager = Balager;