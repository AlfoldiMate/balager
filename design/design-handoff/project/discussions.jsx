// discussions.jsx — threads, sub-threads, votes, pins, closed, attachments
const { useState: useStateD } = React;

function VoteBox({ up, down }) {
  const [v, setV] = useStateD(0);
  const upN = (up || 0) + (v === 1 ? 1 : 0);
  const downN = (down || 0) + (v === -1 ? 1 : 0);
  return (
    <div className="ds-vote">
      <button className={'vb up' + (v === 1 ? ' on' : '')} onClick={() => setV((x) => x === 1 ? 0 : 1)} title="Egyetértek">
        <Icon name="arrowup" size={15} stroke={2.4} /> <span className="vc">{upN}</span>
      </button>
      <button className={'vb down' + (v === -1 ? ' on' : '')} onClick={() => setV((x) => x === -1 ? 0 : -1)} title="Nem értek egyet">
        <Icon name="arrowdown" size={15} stroke={2.4} /> <span className="vc">{downN}</span>
      </button>
    </div>);

}

function PollCard({ poll }) {
  const [picks, setPicks] = useStateD(() => {
    const init = {};
    poll.options.forEach((o) => { init[o.id] = o.votes.includes(ME); });
    return init;
  });
  const toggle = (oid) => setPicks((p) => {
    if (poll.mode === 'single') {
      const n = {}; poll.options.forEach((o) => n[o.id] = false); n[oid] = !p[oid]; return n;
    }
    return { ...p, [oid]: !p[oid] };
  });
  const counts = {};
  let max = 1;
  poll.options.forEach((o) => {
    const base = o.votes.filter((u) => u !== ME).length;
    counts[o.id] = base + (picks[o.id] ? 1 : 0);
    if (counts[o.id] > max) max = counts[o.id];
  });
  const total = Object.values(counts).reduce((a, b) => a + b, 0);
  return (
    <div className="ds-poll">
      <div className="ds-poll-head">
        <Icon name={poll.type === 'date' ? 'calendar' : 'list'} size={15} stroke={2.2} />
        <span className="q">{poll.question}</span>
        <span className="ds-poll-tag">{poll.mode === 'single' ? 'Egy választás' : 'Több is választható'}</span>
      </div>
      <div className="ds-poll-opts">
        {poll.options.map((o) => {
          const on = picks[o.id];
          const c = counts[o.id];
          const voters = o.votes.filter((u) => u !== ME).concat(on ? [ME] : []);
          return (
            <button key={o.id} className={'ds-poll-opt' + (on ? ' on' : '')} onClick={() => toggle(o.id)}>
              <span className={'ds-poll-ctrl' + (poll.mode === 'single' ? ' radio' : '') + (on ? ' on' : '')}>
                {on && <Icon name={poll.mode === 'single' ? 'checkmini' : 'checkmini'} size={11} stroke={3} />}
              </span>
              <span className="ds-poll-bar" style={{ width: (c / max * 100) + '%' }} />
              <span className="ds-poll-label">
                {o.label}{o.sub && <span className="sub"> · {o.sub}</span>}
              </span>
              <span className="ds-poll-voters">
                {voters.length > 0 && <AvatarStack ids={voters} size="sm" max={3} />}
                <span className="n">{c}</span>
              </span>
            </button>);
        })}
      </div>
      <div className="ds-poll-foot">{total} szavazat · a részvétel látható mindenkinek</div>
    </div>);
}

function Message({ msg, isReply }) {
  if (msg.system) {
    return <div className="ds-sys"><Icon name="info" size={14} /> {msg.text} · {msg.time}</div>;
  }
  return (
    <div className={'ds-msg' + (msg.pinned ? ' pinned' : '')}>
      <Avatar id={msg.author} />
      <div className="body">
        <div className="ds-bubble">
          <div className="ds-mhead">
            <span className="au">{USERS[msg.author].name}</span>
            {USERS[msg.author].role === 'approver' && <span className="bg-chip reed" style={{ height: 19, fontSize: 11, padding: '0 7px' }}>Engedélyező</span>}
            <span className="tm">{msg.time}</span>
            {msg.pinned && <span className="pinflag"><Icon name="pin" size={13} fill /> Kitűzve</span>}
          </div>
          {msg.poll ? <PollCard poll={msg.poll} /> : <div className="ds-mtext">{msg.text}</div>}
          {msg.image &&
          <div className="ph-img" style={{ height: 130, marginTop: 10 }}>napozóágy — termékfotó</div>
          }
        </div>
        <div className="ds-mactions">
          {!msg.poll && <VoteBox up={msg.votes} down={msg.down || 0} />}
          <button className="ds-tinybtn"><Icon name="reply" size={15} /> Válasz</button>
          <button className="ds-tinybtn"><Icon name="pin" size={15} /> {msg.pinned ? 'Levesz' : 'Kitűz'}</button>
        </div>
        {msg.replies && msg.replies.length > 0 &&
        <div className="ds-replies">
            {msg.replies.map((r) => <Message key={r.id} msg={r} isReply />)}
          </div>
        }
      </div>
    </div>);

}

function PollBuilder({ onCancel }) {
  const [type, setType] = useStateD('date');
  const [mode, setMode] = useStateD('single');
  const [opts, setOpts] = useStateD(['', '']);
  return (
    <div className="ds-pollbuild">
      <div className="pb-head">
        <Icon name="list" size={16} stroke={2.2} /> <b>Új szavazás</b>
        <button className="bg-iconbtn" style={{ marginLeft: 'auto', width: 32, height: 32 }} onClick={onCancel}><Icon name="x" size={16} /></button>
      </div>
      <input className="bg-input" placeholder="Kérdés — pl. Melyik hétvégén?" />
      <div className="set-pwgrid">
        <div className="bg-field">
          <label>Típus</label>
          <div className="bg-seg">
            <button className={type === 'date' ? 'on' : ''} onClick={() => setType('date')}><Icon name="calendar" size={14} /> Dátum</button>
            <button className={type === 'list' ? 'on' : ''} onClick={() => setType('list')}><Icon name="list" size={14} /> Lista</button>
          </div>
        </div>
        <div className="bg-field">
          <label>Választás</label>
          <div className="bg-seg">
            <button className={mode === 'single' ? 'on' : ''} onClick={() => setMode('single')}>Egy</button>
            <button className={mode === 'multi' ? 'on' : ''} onClick={() => setMode('multi')}>Több</button>
          </div>
        </div>
      </div>
      <div className="bg-field">
        <label>Opciók</label>
        <div className="pb-opts">
          {opts.map((o, i) => (
            <div className="pb-optrow" key={i}>
              <span className={'ds-poll-ctrl' + (mode === 'single' ? ' radio' : '')} />
              <input className="bg-input" placeholder={type === 'date' ? 'pl. Szombat, máj. 30.' : 'Opció szövege…'} defaultValue={o} />
              {opts.length > 2 && <button className="bg-iconbtn" style={{ width: 34, height: 34 }} onClick={() => setOpts((p) => p.filter((_, j) => j !== i))}><Icon name="x" size={15} /></button>}
            </div>
          ))}
        </div>
        <button className="bg-btn ghost sm" style={{ marginTop: 9 }} onClick={() => setOpts((p) => [...p, ''])}><Icon name="plus" size={14} /> Opció hozzáadása</button>
      </div>
      <div style={{ display: 'flex', gap: 10 }}>
        <button className="bg-btn" style={{ flex: 1, justifyContent: 'center' }} onClick={onCancel}><Icon name="check" size={16} /> Szavazás indítása</button>
        <button className="bg-btn ghost" onClick={onCancel}>Mégse</button>
      </div>
    </div>);
}

function ThreadView({ thread, device, onBack, onOpenLink }) {
  const [menu, setMenu] = useStateD(false);
  const [composeMode, setComposeMode] = useStateD('msg'); // 'msg' | 'poll'
  const kindMeta = {
    reservation: { icon: 'calendar', label: 'Foglalás', cls: 'closed' },
    task: { icon: 'tasks', label: 'Feladat', cls: 'reed' },
    general: { icon: 'chat', label: 'Általános', cls: '' }
  }[thread.kind];
  return (
    <div className="ds-thread-view bg-fade">
      <div className="ds-thead">
        {device === 'mobile' && <button className="bg-iconbtn" onClick={onBack} style={{ flexShrink: 0 }}><Icon name="chevleft" size={18} /></button>}
        <div style={{ flex: 1, minWidth: 0 }}>
          <h2>{thread.title}</h2>
          <div className="ds-tmeta">
            <span className="bg-chip"><Avatar id={thread.author} size="sm" /> {USERS[thread.author].name}</span>
            {thread.linkId &&
            <button className={'bg-chip linkchip ' + kindMeta.cls} data-comment-anchor="9c189b5960-span-70-15" onClick={() => onOpenLink && onOpenLink(thread.kind, thread.linkId)} title={(thread.kind === 'task' ? 'Ugrás a feladatra' : 'Ugrás a foglalásra')}><Icon name="link" size={13} /> <span className="lc-txt">{kindMeta.label}: {thread.linkLabel}</span> <Icon name="chevright" size={13} /></button>
            }
            {thread.closed && <span className="bg-chip reject"><Icon name="lock" size={13} /> Lezárva</span>}
          </div>
        </div>
        <div className="ds-menuwrap">
          <button className={'bg-iconbtn' + (menu ? ' on' : '')} style={{ flexShrink: 0 }} onClick={() => setMenu((m) => !m)}><Icon name="dots" size={18} /></button>
          {menu &&
          <React.Fragment>
            <div className="ds-menu-scrim" onClick={() => setMenu(false)} />
            <div className="ds-menu">
              <button className="ds-menu-item"><Icon name="pin" size={16} /> Téma kitűzése</button>
              <button className="ds-menu-item"><Icon name="bell" size={16} /> Némítás</button>
              <button className="ds-menu-item"><Icon name="link" size={16} /> Megosztás…</button>
              <button className="ds-menu-item"><Icon name="lock" size={16} /> Beszélgetés lezárása</button>
              <div className="ds-menu-sep" />
              <button className="ds-menu-item danger"><Icon name="x" size={16} /> Törlés</button>
            </div>
          </React.Fragment>}
        </div>
      </div>
      <div className="ds-scroll">
        {thread.messages.map((m) => <Message key={m.id} msg={m} />)}
      </div>
      {thread.closed ?
      <div className="ds-composer" style={{ justifyContent: 'center', color: 'var(--ink-3)', fontSize: 13.5, fontWeight: 600, gap: 8 }}>
          <Icon name="lock" size={15} /> Ezt a beszélgetést lezárták — csak engedélyező nyithatja meg újra.
        </div> :
      composeMode === 'poll' ?
      <PollBuilder onCancel={() => setComposeMode('msg')} /> :
      <div className="ds-composer">
          <Avatar id={ME} />
          <textarea className="bg-input ds-input" placeholder="Írj egy üzenetet…" />
          <button className="bg-iconbtn ds-pollbtn" title="Szavazás létrehozása" onClick={() => setComposeMode('poll')}><Icon name="list" size={18} /></button>
          <button className="bg-btn" style={{ flexShrink: 0, width: 44, padding: 0, justifyContent: 'center' }}><Icon name="send" size={18} fill /></button>
        </div>
      }
    </div>);

}

function DiscussionsTool({ device, initialThreadId, onOpenLink }) {
  const [filter, setFilter] = useStateD('all');
  const [activeId, setActiveId] = useStateD(initialThreadId || (device === 'mobile' ? null : THREADS[0].id));
  const list = THREADS.filter((t) => filter === 'all' || t.kind === filter);
  const active = THREADS.find((t) => t.id === activeId);
  const kindIcon = { reservation: 'calendar', task: 'tasks', general: 'chat' };

  return (
    <div className={'bg-content'} style={{ overflow: 'hidden', padding: 0 }}>
      <div className={'ds-cols' + (device === 'mobile' && active ? ' show-thread' : '')}>
        <div className="ds-list">
          <ToolHeader
            left={<React.Fragment>
              <div className="ds-filter-tabs">
                {THREAD_FILTERS.map((f) =>
                <button key={f.id} className={filter === f.id ? 'on' : ''} onClick={() => setFilter(f.id)}>{f.label}</button>
                )}
              </div>
              <button className="ds-newthread" title="Új beszélgetés"><Icon name="plus" size={17} /></button>
            </React.Fragment>}
            right={<span className="bg-chip"><Icon name="chat" size={14} /> {list.length} téma</span>}
          />
          {list.map((t) =>
          <div key={t.id} className={'ds-thread' + (t.id === activeId && device !== 'mobile' ? ' active' : '')} onClick={() => setActiveId(t.id)}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <Icon name={kindIcon[t.kind]} size={15} stroke={2} style={{ color: 'var(--water)', flexShrink: 0 }} />
                <span className="tt" style={{ flex: 1 }}>{t.title}</span>
                {t.closed && <Icon name="lock" size={14} style={{ color: 'var(--ink-3)' }} />}
              </div>
              <div className="ex">{t.excerpt}</div>
              <div className="mt">
                <Avatar id={t.author} size="sm" />
                <span>{USERS[t.author].name}</span>
                <span>· {t.time}</span>
                <span style={{ marginLeft: 'auto', display: 'flex', gap: 10 }}>
                  <span style={{ display: 'flex', alignItems: 'center', gap: 4 }}><Icon name="chat" size={13} /> {t.replies}</span>
                  <span style={{ display: 'flex', alignItems: 'center', gap: 4 }}><Icon name="arrowup" size={13} /> {t.votes}</span>
                </span>
              </div>
            </div>
          )}
        </div>
        {active && <ThreadView thread={active} device={device} onBack={() => setActiveId(null)} onOpenLink={onOpenLink} />}
      </div>
    </div>);

}

window.DiscussionsTool = DiscussionsTool;