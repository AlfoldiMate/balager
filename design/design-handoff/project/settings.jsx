// settings.jsx — user settings: notification preferences (e-mail + PWA push)
const { useState: useStateSet } = React;

function SettingsView({ device, onBack }) {
  const init = {};
  NOTIF_GROUPS.forEach((g) => g.rows.forEach((r) => {init[r.id] = { email: r.email, push: r.push };}));
  const [prefs, setPrefs] = useStateSet(init);
  const [profileOpen, setProfileOpen] = useStateSet(false);
  const toggle = (id, ch) => setPrefs((p) => ({ ...p, [id]: { ...p[id], [ch]: !p[id][ch] } }));

  return (
    <div className="bg-content bg-fade" data-comment-anchor="6d550eb043-div-11-5">
      <div className="set-wrap">
        <button className="set-back" onClick={onBack}><Icon name="chevleft" size={16} /> Vissza</button>

        <div className="bg-card set-profcard">
          <button className="set-profhead" onClick={() => setProfileOpen((o) => !o)}>
            <Avatar id={ME} size="lg" ring={false} />
            <div style={{ flex: 1, textAlign: 'left' }}>
              <div className="nm">{USERS[ME].name}</div>
              <div className="em">csaba@balager.hu · Családtag</div>
            </div>
            <Icon name="chevdown" size={18} style={{ color: 'var(--ink-3)', transform: profileOpen ? 'rotate(180deg)' : 'none', transition: 'transform .18s' }} />
          </button>
          {profileOpen &&
          <div className="set-profedit">
            <div className="set-avedit">
              <div className="set-avwrap"><Avatar id={ME} size="lg" ring={false} /><span className="set-avcam"><Icon name="camera" size={14} stroke={2} /></span></div>
              <div>
                <button className="bg-btn ghost sm"><Icon name="camera" size={15} /> Profilkép módosítása</button>
                <p style={{ fontSize: 12, color: 'var(--ink-3)', marginTop: 6 }}>JPG vagy PNG, max. 2 MB.</p>
              </div>
            </div>
            <div className="bg-field"><label>Név</label><input className="bg-input" defaultValue="Csaba" /></div>
            <div className="bg-field"><label>E-mail cím</label><input className="bg-input" type="email" defaultValue="csaba@balager.hu" /></div>
            <div className="set-pwgrid">
              <div className="bg-field"><label>Jelenlegi jelszó</label><input className="bg-input" type="password" defaultValue="balaton26" /></div>
              <div className="bg-field"><label>Új jelszó</label><input className="bg-input" type="password" placeholder="Új jelszó…" /></div>
            </div>
            <div style={{ display: 'flex', gap: 10 }}>
              <button className="bg-btn" onClick={() => setProfileOpen(false)}><Icon name="check" size={16} /> Mentés</button>
              <button className="bg-btn ghost" onClick={() => setProfileOpen(false)}>Mégse</button>
            </div>
          </div>}
        </div>

        <div className="set-pwa">
          <div className="pi"><Icon name="phone" size={20} /></div>
          <div className="pt">
            <b>Push értesítések ezen az eszközön</b>
            <p>A Balager telepíthető a kezdőképernyőre (PWA). Az iOS push engedélyezve.</p>
          </div>
          <Switch on={true} onClick={() => {}} />
        </div>

        <div>
          <div style={{ fontSize: 13, fontWeight: 700, color: 'var(--ink-3)', textTransform: 'uppercase', letterSpacing: '.04em', padding: '4px 4px 2px' }}>Értesítési beállítások</div>
          <div className="set-colhead">
            <span><Icon name="mail" size={13} /> E-mail</span>
            <span><Icon name="bell" size={13} /> Push</span>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            {NOTIF_GROUPS.map((g) =>
            <div className="bg-card set-card" key={g.id}>
                <div className="set-grouphead">
                  <div className="gi"><Icon name={g.icon} size={17} stroke={2} /></div>
                  <h4>{g.label}</h4>
                </div>
                {g.rows.map((r) =>
              <div className="set-row" key={r.id}>
                    <div className="rl">
                      <div className="t">{r.label}</div>
                      {r.sub && <div className="s">{r.sub}</div>}
                    </div>
                    <div className="toggles">
                      <div className="tg"><Switch on={prefs[r.id].email} onClick={() => toggle(r.id, 'email')} /></div>
                      <div className="tg"><Switch on={prefs[r.id].push} onClick={() => toggle(r.id, 'push')} /></div>
                    </div>
                  </div>
              )}
              </div>
            )}
          </div>
        </div>

        <button className="bg-btn ghost" style={{ alignSelf: 'flex-start' }}><Icon name="logout" size={16} /> Kijelentkezés</button>
      </div>
    </div>);

}

window.SettingsView = SettingsView;