// info.jsx — house rules & how-to
function InfoTool({ device }) {
  const tiles = [
  { icon: 'calendar', h: 'Foglalások', p: 'Görgethető heti naptár. Jelölj ki szabad napokat és kérj foglalást — két engedélyező hagyja jóvá.' },
  { icon: 'tasks', h: 'Feladatok', p: 'Csoportosított teendők alfeladatokkal, ismétlődéssel. Egyes feladatok nyitott foglalássá válhatnak.' },
  { icon: 'chat', h: 'Beszélgetések', p: 'Nyiss témát bármiről. A foglalások és feladatok eseményei automatikusan ide kerülnek.' },
  { icon: 'bell', h: 'Értesítések', p: 'E-mail és push értesítés a fontos eseményekről. A profilodban szabhatod testre.' }];

  const rules = [
  { lead: 'Foglalás = napok.', t: 'Kezdő és záró időpont nincs, csak teljes napok. Egy hétvége jellemzően péntektől vasárnapig tart.' },
  { lead: 'Két jóváhagyás kell.', t: 'Egy foglalás akkor elfogadott, ha minden engedélyező (Anna és Béla) jóváhagyta. Bárki elutasíthatja, indoklással.' },
  { lead: 'Zárt vs. nyitott.', t: 'Zárt foglalásnál csak a foglaló kezeli a résztvevőket. Nyitott foglaláshoz bárki csatlakozhat.' },
  { lead: 'Hagyd tisztán.', t: 'Távozás előtt: szemét kivihető, hűtő kiürítve, redőnyök leengedve, gázcsap elzárva.' },
  { lead: 'Stég és csónak.', t: 'A mentőmellény kötelező a gyerekeknek. A csónakkulcs a bejárati szekrény felső fiókjában.' }];

  return (
    <div className="bg-content bg-fade">
      <div className="nf-wrap" data-comment-anchor="63050f22b7-div-18-7">
        <div className="nf-hero">
          <div className="sunblob" />
          <div style={{ position: 'relative' }}>
            <div className="bg-chip" style={{ background: 'rgba(255,255,255,.18)', border: 'none', color: '#fff', marginBottom: 12 }}>
              <Icon name="waves" size={14} /> Balaton · Családi nyaraló
            </div>
            <h2>Üdv a Balagerben!</h2>
            <p>Itt egy helyen kezeljük a nyaraló foglalásait, a közös teendőket és minden beszélgetést. Az alábbiakban a legfontosabb tudnivalók és a házirend.</p>
          </div>
        </div>

        <div className="nf-grid">
          {tiles.map((t) =>
          <div className="bg-card nf-tile" key={t.h}>
              <div className="ic"><Icon name={t.icon} size={20} /></div>
              <h4>{t.h}</h4>
              <p>{t.p}</p>
            </div>
          )}
        </div>

        <div className="bg-card nf-rules">
          <h4>Házirend és tudnivalók</h4>
          {rules.map((r, i) =>
          <div className="nf-rule" key={i}>
              <span className="no">{String(i + 1).padStart(2, '0')}</span>
              <p><span className="lead">{r.lead}</span> {r.t}</p>
            </div>
          )}
        </div>

        <div className="bg-card nf-rules" style={{ display: 'flex', gap: 18, alignItems: 'center', flexWrap: 'wrap' }}>
          <div className="ph-img" style={{ width: device === 'mobile' ? '100%' : 220, height: 130, flexShrink: 0 }}>nyaraló — fotó</div>
          <div style={{ flex: 1, minWidth: 180 }}>
            <h4 style={{ marginBottom: 6 }}>Cím és megközelítés</h4>
            <p style={{ fontSize: 14, color: 'var(--ink-2)' }}>8638 Balatonlelle, Nád utca 7.</p>
            <button className="bg-btn ghost sm" style={{ marginTop: 12 }}><Icon name="map" size={15} /> Térkép megnyitása</button>
          </div>
        </div>
      </div>
    </div>);

}

window.InfoTool = InfoTool;