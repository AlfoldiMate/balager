// reservations.jsx — continuous scrollable week calendar + detail/new panel
const { useState: useStateR, useMemo: useMemoR, useRef: useRefR, useEffect: useEffectR } = React;

function ReservationPanel({ mode, res, selection, device, onClose, onAccessChange, onGoDiscuss }) {
  // NEW reservation form
  if (mode === 'new') {
    const days = selection ? (Math.round((new Date(selection.end) - new Date(selection.start)) / 86400000) + 1) : 0;
    const [access, setAccess] = useStateR('closed');
    return (
      <aside className="bg-panel">
        {device === 'mobile' && <div className="sheet-grab" />}
        <div className="ph">
          <div style={{ flex: 1 }}>
            <h3>Új foglalás</h3>
            <div style={{ fontSize: 13, color: 'var(--ink-3)', marginTop: 3 }}>{selection && huRange(selection.start, selection.end)} · {days} nap</div>
          </div>
          <button className="bg-iconbtn" onClick={onClose}><Icon name="x" size={18} /></button>
        </div>
        <div className="pbody">
          <div className="bg-field">
            <label>Foglalás neve</label>
            <input className="bg-input" defaultValue="Hétvége a háznál" />
          </div>
          <div className="bg-field">
            <label>Hozzáférés</label>
            <div className="bg-seg">
              <button className={access === 'closed' ? 'on closed' : ''} onClick={() => setAccess('closed')}>
                <Icon name="lock" size={15} stroke={2.2} /> Zárt
              </button>
              <button className={access === 'open' ? 'on open' : ''} onClick={() => setAccess('open')}>
                <Icon name="users" size={15} stroke={2.2} /> Nyitott
              </button>
            </div>
            <p style={{ fontSize: 12.5, color: 'var(--ink-3)', marginTop: 8 }}>
              {access === 'closed'
                ? 'Csak te kezelheted a résztvevőket.'
                : 'Bármely családtag csatlakozhat ehhez a hétvégéhez.'}
            </p>
          </div>
          <div className="bg-field">
            <label>Üzenet az engedélyezőknek</label>
            <textarea className="bg-input" rows={3} placeholder="Pl. nyugis hétvége a tónál…" style={{ resize: 'none' }} />
          </div>
          <div className="bg-field">
            <label>Engedélyezés szükséges</label>
            <div className="appr">
              {['anna', 'bela'].map(id => (
                <div className="appr-row" key={id}>
                  <Avatar id={id} size="sm" />
                  <span className="nm">{USERS[id].name}</span>
                  <span className="bg-chip" style={{ marginLeft: 'auto' }}><Icon name="clock" size={13} /> Vár</span>
                </div>
              ))}
            </div>
          </div>
        </div>
        <div className="pfoot">
          <button className="bg-btn ghost" onClick={onClose}>Mégse</button>
          <button className="bg-btn" style={{ flex: 1, justifyContent: 'center' }} onClick={onClose}>
            <Icon name="check" size={17} /> Foglalás kérése
          </button>
        </div>
      </aside>
    );
  }

  // VIEW reservation
  if (!res) return null;
  const m = STATUS_META[res.status];
  const isOwner = res.owner === ME;
  const isOpen = res.status === 'open';
  return (
    <aside className="bg-panel">
      {device === 'mobile' && <div className="sheet-grab" />}
      <div className="ph">
        <div style={{ flex: 1 }}>
          <div style={{ marginBottom: 7 }}><StatusChip status={res.status} /></div>
          <h3>{res.title}</h3>
          <div style={{ fontSize: 13, color: 'var(--ink-3)', marginTop: 4, display: 'flex', alignItems: 'center', gap: 6 }}>
            <Icon name="calendar" size={14} /> {huRange(res.from, res.to)}
          </div>
        </div>
        <button className="bg-iconbtn" onClick={onClose}><Icon name="x" size={18} /></button>
      </div>
      <div className="pbody">
        {res.status === 'reject' && res.rejectReason && (
          <div style={{ padding: '12px 14px', borderRadius: 11, background: 'var(--st-reject-bg)', border: '1px solid var(--st-reject-bd)', color: 'var(--st-reject-ink)', fontSize: 13.5 }}>
            <b style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 4 }}><Icon name="x" size={14} /> Elutasítva — {USERS.anna.name}</b>
            {res.rejectReason}
          </div>
        )}
        {res.note && (
          <div style={{ padding: '12px 14px', borderRadius: 11, background: 'var(--surface-2)', border: '1px solid var(--line)', fontSize: 13.5, color: 'var(--ink-2)' }}>
            {res.note}
          </div>
        )}

        <div className="bg-field">
          <label>Foglaló</label>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            <Avatar id={res.owner} size="sm" />
            <span style={{ fontWeight: 600 }}>{USERS[res.owner].name}</span>
            {USERS[res.owner].role === 'approver' && <span className="bg-chip reed" style={{ height: 22 }}>Engedélyező</span>}
          </div>
        </div>

        <div className="bg-field">
          <label>Hozzáférés</label>
          <div className="bg-seg">
            <button className={!isOpen ? 'on closed' : ''} onClick={() => isOwner && onAccessChange(res, 'closed')} disabled={!isOwner}>
              <Icon name="lock" size={15} stroke={2.2} /> Zárt
            </button>
            <button className={isOpen ? 'on open' : ''} onClick={() => isOwner && onAccessChange(res, 'open')} disabled={!isOwner}>
              <Icon name="users" size={15} stroke={2.2} /> Nyitott
            </button>
          </div>
          {!isOwner && <p style={{ fontSize: 12, color: 'var(--ink-3)', marginTop: 7 }}>A hozzáférést csak a foglaló módosíthatja.</p>}
        </div>

        <div className="bg-field">
          <label>Résztvevők · {res.attendees.length}</label>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 7 }}>
            {res.attendees.map(id => (
              <div key={id} style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                <Avatar id={id} size="sm" />
                <span style={{ fontWeight: 500, fontSize: 14 }}>{USERS[id].name}</span>
                {id === res.owner && <span style={{ fontSize: 12, color: 'var(--ink-3)', marginLeft: 'auto' }}>foglaló</span>}
              </div>
            ))}
          </div>
          {isOpen && res.owner !== ME && !res.attendees.includes(ME) && (
            <button className="bg-btn sun sm" style={{ marginTop: 11 }}><Icon name="plus" size={15} /> Csatlakozom</button>
          )}
        </div>

        <div className="bg-field">
          <label>Engedélyezés</label>
          <div className="appr">
            {Object.entries(res.approvals).map(([id, st]) => (
              <div className="appr-row" key={id}>
                <Avatar id={id} size="sm" />
                <span className="nm">{USERS[id].name}</span>
                <span className="st">
                  {st === 'approved' && <span className="bg-chip open" style={{ height: 22 }}><Icon name="check" size={13} /> Jóváhagyta</span>}
                  {st === 'rejected' && <span className="bg-chip reject" style={{ height: 22 }}><Icon name="x" size={13} /> Elutasította</span>}
                  {st === 'pending' && <span className="bg-chip" style={{ height: 22 }}><Icon name="clock" size={13} /> Vár</span>}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>
      <div className="pfoot">
        <button className="bg-btn ghost" style={{ flex: 1, justifyContent: 'center' }} onClick={() => onGoDiscuss(res)}>
          <Icon name="chat" size={16} /> Beszélgetés
        </button>
        {isOwner && <button className="bg-btn ghost" style={{ width: 44, justifyContent: 'center', padding: 0 }}><Icon name="dots" size={18} /></button>}
      </div>
    </aside>
  );
}

function ReservationsTool({ device, onGoDiscuss, focusRes }) {
  const [sel, setSel] = useStateR(null);          // {start, end}
  const [panel, setPanel] = useStateR(null);       // {mode:'view'|'new', res}
  const [accessOverride, setAccessOverride] = useStateR({});

  useEffectR(() => {
    if (!focusRes) return;
    const r = RESERVATIONS.find((x) => x.id === focusRes.id);
    if (r) { setSel(null); setPanel({ mode: 'view', res: r }); }
  }, [focusRes && focusRes.n]);

  const months = useMemoR(() => {
    const out = [];
    for (let m = 4; m <= 8; m++) {            // máj → szept 2026
      const year = 2026;
      const daysInMonth = new Date(year, m + 1, 0).getDate();
      const lead = (new Date(year, m, 1).getDay() + 6) % 7;  // Monday-based blanks
      const weeks = [];
      let week = new Array(7).fill(null);
      let col = lead;
      for (let day = 1; day <= daysInMonth; day++) {
        week[col] = new Date(year, m, day);
        if (++col === 7) { weeks.push(week); week = new Array(7).fill(null); col = 0; }
      }
      if (col > 0) weeks.push(week);
      out.push({ year, month: m, weeks });
    }
    return out;
  }, []);
  const ICONMAP = { pending: 'clock', reject: 'x', closed: 'lock', open: 'users' };

  function statusOf(r) { return accessOverride[r.id] || r.status; }

  function clickDay(d) {
    const di = iso(d);
    if (!sel) setSel({ start: di, end: di });
    else if (di < sel.start) setSel({ start: di, end: sel.end });
    else if (di > sel.end) setSel({ start: sel.start, end: di });
    else setSel({ start: di, end: di });
    // keep the 'new' reservation form open while picking dates on the calendar
    if (panel && panel.mode !== 'new') setPanel(null);
  }

  function inSel(di) { return sel && di >= sel.start && di <= sel.end; }
  const selDays = sel ? Math.round((new Date(sel.end) - new Date(sel.start)) / 86400000) + 1 : 0;

  const todayIso = iso(TODAY);
  const scrollRef = useRefR(null);
  const todayRef = useRefR(null);
  function scrollToToday() {
    const c = scrollRef.current, t = todayRef.current;
    if (c && t) c.scrollTop += t.getBoundingClientRect().top - c.getBoundingClientRect().top - 96;
  }
  useEffectR(() => { const id = requestAnimationFrame(scrollToToday); return () => cancelAnimationFrame(id); }, []);

  return (
    <React.Fragment>
      <div className="bg-content bg-fade" ref={scrollRef}>
        <div className="cal-wrap">
          <ToolHeader
            left={<React.Fragment>
              <button className="bg-btn ghost" onClick={scrollToToday} title="Ugrás a mai napra"><Icon name="calendar" size={16} stroke={2.2} /> Ma</button>
              <span className="cal-hint">Húzz végig napokon át új foglaláshoz</span>
            </React.Fragment>}
            right={<div className="cal-legend">
              {[['pending', 'Függőben'], ['reject', 'Elutasítva'], ['closed', 'Zárt'], ['open', 'Nyitott']].map(([c, l]) => (
                <span className="lg" key={c}><span className="sw" style={{ background: `var(--st-${c === 'reject' ? 'reject' : c === 'pending' ? 'pending' : c}-bg)`, borderColor: `var(--st-${c}-bd)` }} /> {l}</span>
              ))}
            </div>}
          />

          <div className="cal-grid">
            <div className="cal-head">
              <div className="wk" />
              {HU_DAYS_SHORT.map((d, i) => <div key={d} className={'dh' + (i >= 5 ? ' we' : '')}>{d}</div>)}
            </div>
            {months.map((mo, mi) => (
              <React.Fragment key={mi}>
                <div className="cal-month-label">{HU_MONTHS[mo.month]} {mo.year}</div>
                {mo.weeks.map((week, wi) => {
                  // merged reservation segments within this week
                  const bars = [];
                  RESERVATIONS.forEach(r => {
                    let s = -1, e = -1;
                    week.forEach((d, ci) => {
                      if (d) { const di = iso(d); if (di >= r.from && di <= r.to) { if (s < 0) s = ci; e = ci; } }
                    });
                    if (s >= 0) bars.push({
                      r, s, e,
                      capL: iso(week[s]) === r.from,
                      capR: iso(week[e]) === r.to,
                    });
                  });
                  const firstDay = week.find(d => d);
                  return (
                    <div className="cal-week" key={wi}>
                      <div className="cal-wknum"><span>hét</span><b>{firstDay ? isoWeek(firstDay) : ''}</b></div>
                      {week.map((d, ci) => {
                        if (!d) return <div key={ci} className="cal-day ph" />;
                        const di = iso(d);
                        const hit = reservationOn(di);
                        const st = hit ? statusOf(hit.res) : null;
                        const cls = ['cal-day'];
                        if (di === todayIso) cls.push('today');
                        if (hit) cls.push('has', st);
                        if (inSel(di)) cls.push('sel-mid');
                        if (sel && (di === sel.start || di === sel.end)) cls.push('sel');
                        return (
                          <div key={ci} ref={di === todayIso ? todayRef : null} className={cls.join(' ')} onClick={() => clickDay(d)}>
                            <div className="dn"><span className="num">{d.getDate()}</span></div>
                          </div>
                        );
                      })}
                      {bars.length > 0 && (
                        <div className="cal-week-bars">
                          {bars.map((b, bi) => {
                            const st = statusOf(b.r);
                            const owner = USERS[b.r.owner];
                            const cls = ['cal-bar', st];
                            if (b.capL) cls.push('cap-l');
                            if (b.capR) cls.push('cap-r');
                            return (
                              <div key={bi} className={cls.join(' ')}
                                style={{ gridColumn: `${b.s + 1} / ${b.e + 2}` }}
                                title={`${b.r.title} · ${owner.name}`}
                                onClick={(ev) => { ev.stopPropagation(); setSel(null); setPanel({ mode: 'view', res: b.r }); }}>
                                {b.capL
                                  ? <React.Fragment>
                                      <Icon name={ICONMAP[st]} size={12} stroke={2.4} style={{ flexShrink: 0 }} />
                                      <span className="bt">{b.r.title}</span>
                                      <span className="cal-owner">{owner.name}</span>
                                    </React.Fragment>
                                  : <span className="bt cont">{b.r.title}</span>}
                              </div>
                            );
                          })}
                        </div>
                      )}
                    </div>
                  );
                })}
              </React.Fragment>
            ))}
          </div>

          {sel && (
            <div className="cal-selbar">
              <Icon name="calendar" size={18} />
              <div>
                <div style={{ fontWeight: 700, fontSize: 14 }}>{huRange(sel.start, sel.end)}</div>
                <div style={{ fontSize: 12, color: 'rgba(255,255,255,.65)' }}>{selDays} nap kiválasztva</div>
              </div>
              <button className="bg-btn sun sm" style={{ marginLeft: 8 }} onClick={() => setPanel({ mode: 'new' })}>
                <Icon name="check" size={15} /> Foglalás kérése
              </button>
              <button className="x" onClick={() => setSel(null)}><Icon name="x" size={17} /></button>
            </div>
          )}
        </div>
      </div>

      {panel && (
        <ReservationPanel
          mode={panel.mode}
          res={panel.res}
          selection={sel}
          device={device}
          onClose={() => { setPanel(null); if (panel.mode === 'new') setSel(null); }}
          onAccessChange={(r, acc) => setAccessOverride(o => ({ ...o, [r.id]: acc }))}
          onGoDiscuss={onGoDiscuss}
        />
      )}
    </React.Fragment>
  );
}

window.ReservationsTool = ReservationsTool;
