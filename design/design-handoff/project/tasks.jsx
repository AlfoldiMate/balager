// tasks.jsx — grouped tasks, subtasks, recurring, attached events, editor panel
const { useState: useStateT } = React;

function SubRow({ sub }) {
  const [done, setDone] = useStateT(sub.done);
  return (
    <div className={'tk-subrow' + (done ? ' done' : '')}>
      <button className={'tk-check' + (done ? ' done' : '')} onClick={() => setDone((d) => !d)}>
        {done && <Icon name="checkmini" size={11} stroke={2.6} />}
      </button>
      <span className="tx" style={{ flex: 1 }}>{sub.title}</span>
    </div>);
}

function TaskMenu({ onClose }) {
  return (
    <React.Fragment>
      <div className="ds-menu-scrim" onClick={onClose} />
      <div className="ds-menu" onClick={(e) => e.stopPropagation()}>
        <button className="ds-menu-item" onClick={onClose}><Icon name="check" size={16} /> Késznek jelöl</button>
        <button className="ds-menu-item" onClick={onClose}><Icon name="users" size={16} /> Áthelyezés máshoz</button>
        <button className="ds-menu-item" onClick={onClose}><Icon name="repeat" size={16} /> Ismétlődés beállítása</button>
        <button className="ds-menu-item" onClick={onClose}><Icon name="folder" size={16} /> Áthelyezés csoportba</button>
        <div className="ds-menu-sep" />
        <button className="ds-menu-item danger" onClick={onClose}><Icon name="x" size={16} /> Törlés</button>
      </div>
    </React.Fragment>);
}

function TaskItem({ task, device, onGoDiscuss, onOpenEvent, onEdit }) {
  const [done, setDone] = useStateT(task.done);
  const [open, setOpen] = useStateT(false);
  const [menu, setMenu] = useStateT(false);
  const subDone = task.subs.filter((s) => s.done).length;
  const hasSubs = task.subs.length > 0;
  const stop = (e) => e.stopPropagation();
  return (
    <div className={'tk-item' + (done ? ' done' : '')}>
      <div className="tk-row tk-clickable" onClick={() => onEdit(task)}>
        <button className={'tk-check' + (done ? ' done' : '')} onClick={(e) => { stop(e); setDone((d) => !d); }}>
          {done && <Icon name="check" size={14} stroke={2.6} />}
        </button>
        <div className="tk-main">
          <div className="tk-title">
            <span className="txt">{task.title}</span>
            {task.recurring && <span className="bg-chip reed" style={{ height: 22 }}><Icon name="repeat" size={13} /> {task.recurring}</span>}
          </div>
          <div className="tk-meta">
            <span className="m"><Avatar id={task.assignee} size="sm" /> {USERS[task.assignee].name}</span>
            {task.due && <span className="m"><Icon name="clock" /> {huDate(new Date(task.due + 'T00:00'))}</span>}
            {hasSubs &&
            <button className="m" style={{ cursor: 'pointer' }} onClick={(e) => { stop(e); setOpen((o) => !o); }}>
                <Icon name="list" /> {subDone}/{task.subs.length} alfeladat
                <Icon name="chevdown" size={13} style={{ transform: open ? 'rotate(180deg)' : 'none', transition: 'transform .18s' }} />
              </button>
            }
            <button className="m tk-discuss" onClick={(e) => { stop(e); onGoDiscuss(task); }}><Icon name="chat" /> Beszélgetés</button>
          </div>
        </div>
        {task.event &&
        <button className="tk-eventchip" onClick={(e) => { stop(e); onOpenEvent && onOpenEvent(task.event.resId); }} title={'Eseményhez kötve: ' + task.event.label}>
          <Icon name="calendar" size={15} stroke={2.2} />
        </button>
        }
        <div className="tk-menuwrap" onClick={stop}>
          <button className={'bg-iconbtn tk-dots' + (menu ? ' on' : '')} onClick={() => setMenu((m) => !m)}><Icon name="dots" size={17} /></button>
          {menu && <TaskMenu onClose={() => setMenu(false)} />}
        </div>
      </div>
      {hasSubs && open &&
      <div className="tk-sub">
          {task.subs.map((s) => <SubRow key={s.id} sub={s} />)}
        </div>
      }
    </div>);
}

function TaskGroup({ group, tasks, device, onGoDiscuss, onOpenEvent, onEdit, open, onToggle }) {
  const openCount = group.tasks.filter((t) => !t.done).length;
  return (
    <div className={'tk-group' + (open ? '' : ' closed')}>
      <button className="tk-ghead" onClick={onToggle}>
        <Icon name="chevdown" size={16} stroke={2.2} className="gchev" />
        <Icon name="folder" size={17} stroke={2} style={{ color: 'var(--water)' }} />
        <span className="gname">{group.name}</span>
        <span className="gcount">{openCount}/{group.tasks.length}</span>
        <span className="gline" />
      </button>
      {open &&
      <div className="tk-list">
          {tasks.map((t) => <TaskItem key={t.id} task={t} device={device} onGoDiscuss={onGoDiscuss} onOpenEvent={onOpenEvent} onEdit={onEdit} />)}
        </div>
      }
    </div>);
}

function TaskPanel({ task, device, onClose, onGoDiscuss, onOpenEvent }) {
  const isNew = !task.id;
  const [done, setDone] = useStateT(!!task.done);
  const [recur, setRecur] = useStateT(task.recurring || '');
  const group = TASK_GROUPS.find((g) => g.tasks.some((t) => t.id === task.id));
  return (
    <aside className="bg-panel">
      {device === 'mobile' && <div className="sheet-grab" />}
      <div className="ph">
        <div style={{ flex: 1 }}>
          <div style={{ marginBottom: 7, display: 'flex', gap: 7 }}>
            <span className="bg-chip"><Icon name="folder" size={13} /> {group ? group.name : 'Új feladat'}</span>
            {done && <span className="bg-chip open" style={{ height: 26 }}><Icon name="check" size={13} /> Kész</span>}
          </div>
          <h3>{isNew ? 'Új feladat' : 'Feladat szerkesztése'}</h3>
        </div>
        <button className="bg-iconbtn" onClick={onClose}><Icon name="x" size={18} /></button>
      </div>
      <div className="pbody">
        <div className="bg-field">
          <label>Megnevezés</label>
          <input className="bg-input" defaultValue={task.title} placeholder="Mi a teendő?" />
        </div>
        <div className="set-pwgrid">
          <div className="bg-field">
            <label>Felelős</label>
            <div className="bg-input" style={{ display: 'flex', alignItems: 'center', gap: 9 }}>
              <Avatar id={task.assignee || ME} size="sm" /> {USERS[task.assignee || ME].name}
              <Icon name="chevdown" size={15} style={{ marginLeft: 'auto', color: 'var(--ink-3)' }} />
            </div>
          </div>
          <div className="bg-field">
            <label>Határidő</label>
            <div className="bg-input" style={{ display: 'flex', alignItems: 'center', gap: 9 }}>
              <Icon name="clock" size={15} style={{ color: 'var(--ink-3)' }} />
              {task.due ? huDate(new Date(task.due + 'T00:00')) : 'Nincs'}
            </div>
          </div>
        </div>
        <div className="bg-field">
          <label>Ismétlődés</label>
          <div className="bg-seg">
            {['', 'Hetente', 'Kéthetente', 'Havonta'].map((r) => (
              <button key={r || 'once'} className={recur === r ? 'on' : ''} onClick={() => setRecur(r)}>{r || 'Egyszeri'}</button>
            ))}
          </div>
        </div>
        {task.subs && task.subs.length > 0 &&
        <div className="bg-field">
          <label>Alfeladatok · {task.subs.filter((s) => s.done).length}/{task.subs.length}</label>
          <div className="tk-paneledit-subs">{task.subs.map((s) => <SubRow key={s.id} sub={s} />)}</div>
          <button className="bg-btn ghost sm" style={{ marginTop: 9 }}><Icon name="plus" size={14} /> Alfeladat</button>
        </div>
        }
        {task.event &&
        <div className="bg-field">
          <label>Kapcsolt esemény</label>
          <button className="tk-event" style={{ margin: 0, width: '100%' }} onClick={() => onOpenEvent && onOpenEvent(task.event.resId)}>
            <Icon name="calendar" size={15} stroke={2.2} /> <b>{task.event.label}</b> · nyitott foglalás
            <Icon name="chevright" size={14} style={{ marginLeft: 'auto' }} />
          </button>
        </div>
        }
      </div>
      <div className="pfoot">
        <button className="bg-btn ghost" style={{ flex: 1, justifyContent: 'center' }} onClick={() => onGoDiscuss(task)}>
          <Icon name="chat" size={16} /> Beszélgetés
        </button>
        <button className="bg-btn" style={{ flex: 1, justifyContent: 'center' }} onClick={onClose}>
          <Icon name="check" size={16} /> Mentés
        </button>
      </div>
    </aside>);
}

function TasksTool({ device, onGoDiscuss, onOpenEvent }) {
  const total = TASK_GROUPS.reduce((a, g) => a + g.tasks.length, 0);
  const open = TASK_GROUPS.reduce((a, g) => a + g.tasks.filter((t) => !t.done).length, 0);
  const [openMap, setOpenMap] = useStateT(() => Object.fromEntries(TASK_GROUPS.map((g) => [g.id, true])));
  const [panelTask, setPanelTask] = useStateT(null);
  const [filter, setFilter] = useStateT('all');
  const anyOpen = Object.values(openMap).some(Boolean);
  const toggleAll = () => setOpenMap(Object.fromEntries(TASK_GROUPS.map((g) => [g.id, !anyOpen])));
  const matchFilter = (t) => filter === 'all' ? true : filter === 'active' ? !t.done : t.done;
  const groups = TASK_GROUPS.map((g) => ({ g, tasks: g.tasks.filter(matchFilter) })).filter((x) => x.tasks.length > 0);
  return (
    <React.Fragment>
      <div className="bg-content bg-fade" data-comment-anchor="d674d01c27-div-66-5">
        <div className="tk-wrap">
          <ToolHeader
            left={<React.Fragment>
              <button className="bg-btn" title="Új feladat" style={{ width: 44, padding: 0, justifyContent: 'center' }} onClick={() => setPanelTask({ assignee: ME, subs: [] })}><Icon name="plus" size={18} /></button>
              <button className="bg-btn ghost" title="Új csoport" style={{ width: 44, padding: 0, justifyContent: 'center' }}><Icon name="folder" size={17} /></button>
              <button className="bg-iconbtn" onClick={toggleAll} title={anyOpen ? 'Összes csoport összecsukása' : 'Összes csoport kinyitása'}>
                <Icon name={anyOpen ? 'collapseall' : 'expandall'} size={18} stroke={2} />
              </button>
            </React.Fragment>}
            right={<div className="hdr-tabs">
              <button className={filter === 'all' ? 'on' : ''} onClick={() => setFilter('all')}>Mind <span className="c">{total}</span></button>
              <button className={filter === 'active' ? 'on' : ''} onClick={() => setFilter('active')}>Aktív <span className="c">{open}</span></button>
              <button className={filter === 'done' ? 'on' : ''} onClick={() => setFilter('done')}>Kész <span className="c">{total - open}</span></button>
            </div>}
          />

          <div className="tk-groups">
            {groups.length === 0 ?
            <div className="tk-empty"><Icon name="check" size={26} stroke={2} /> Nincs ilyen feladat.</div> :
            groups.map(({ g, tasks }) =>
            <TaskGroup key={g.id} group={g} tasks={tasks} device={device} onGoDiscuss={onGoDiscuss} onOpenEvent={onOpenEvent} onEdit={setPanelTask} open={openMap[g.id]} onToggle={() => setOpenMap((m) => ({ ...m, [g.id]: !m[g.id] }))} />
            )}
          </div>
        </div>
      </div>
      {panelTask &&
      <TaskPanel task={panelTask} device={device} onClose={() => setPanelTask(null)} onGoDiscuss={onGoDiscuss} onOpenEvent={onOpenEvent} />
      }
    </React.Fragment>);
}

window.TasksTool = TasksTool;
