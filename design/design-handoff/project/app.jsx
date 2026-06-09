// app.jsx — canvas: desktop + mobile side by side, + Tweaks
const { useState: useStateA } = React;

const WATER_PALETTES = {
  'Tó-kék':      ['#3f8aa3', '#2f6f86', '#e4eff3'],
  'Nád-zöld':    ['#5a9b7c', '#437a5f', '#e3efe7'],
  'Mély víz':    ['#356b9b', '#28547a', '#e2ebf3'],
  'Naplemente':  ['#c47b4a', '#a8623a', '#f6e7da'],
};

const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "water": ["#356b9b", "#28547a", "#e2ebf3"],
  "density": "regular",
  "radius": 14,
  "sidebarOpen": true
}/*EDITMODE-END*/;

function FrameLabel({ icon, children }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 14, color: '#3a524d', fontWeight: 700, fontSize: 14, letterSpacing: '.01em' }}>
      <Icon name={icon} size={17} stroke={2} /> {children}
    </div>
  );
}

function Canvas() {
  const [t, setTweak] = useTweaks(TWEAK_DEFAULTS);
  const [w, wd, ws] = t.water;

  const rootStyle = {
    '--water': w, '--water-deep': wd, '--water-soft': ws,
    '--radius': t.radius + 'px', '--radius-sm': Math.max(4, t.radius - 5) + 'px',
  };
  const densCls = t.density === 'regular' ? '' : ' dens-' + t.density;

  return (
    <div className={'bg-root' + densCls} style={rootStyle}>
      <div style={{
        minHeight: '100vh', background: 'radial-gradient(1200px 600px at 70% -10%, #e8efe9, #d3ded8)',
        padding: '40px 48px 80px', boxSizing: 'border-box',
      }}>
        <header style={{ maxWidth: 1700, margin: '0 auto 30px', display: 'flex', alignItems: 'center', gap: 16 }}>
          <div style={{ width: 46, height: 46, borderRadius: 13, background: 'linear-gradient(150deg,' + wd + ',' + w + ')', display: 'grid', placeItems: 'center', boxShadow: '0 6px 18px rgba(20,48,46,.18)' }}>
            <Icon name="sun" size={26} stroke={2.2} style={{ color: '#fff' }} />
          </div>
          <div>
            <div style={{ fontFamily: 'Hanken Grotesk, sans-serif', fontWeight: 800, fontSize: 24, color: '#16302e', letterSpacing: '-.02em' }}>Balager</div>
            <div style={{ fontSize: 13.5, color: '#5a716b' }}>Családi nyaraló-kezelő a Balatonra — foglalások, feladatok, beszélgetések</div>
          </div>
          <div style={{ marginLeft: 'auto', display: 'flex', gap: 8, fontFamily: 'Spline Sans Mono, monospace', fontSize: 11.5, color: '#5a716b' }}>
            <span style={{ padding: '6px 11px', background: 'rgba(255,255,255,.6)', borderRadius: 8, border: '1px solid rgba(20,48,46,.08)' }}>hi-fi prototípus</span>
            <span style={{ padding: '6px 11px', background: 'rgba(255,255,255,.6)', borderRadius: 8, border: '1px solid rgba(20,48,46,.08)' }}>magyar UI</span>
          </div>
        </header>

        <div style={{ display: 'flex', gap: 56, alignItems: 'flex-start', justifyContent: 'center', flexWrap: 'wrap' }}>
          <div>
            <FrameLabel icon="home">Asztali nézet</FrameLabel>
            <ChromeWindow width={1180} height={788} url="balager.app/foglalasok" tabs={[{ title: 'Balager — Foglalások' }]}>
              <Balager device="desktop" sidebarDefault={t.sidebarOpen} />
            </ChromeWindow>
          </div>
          <div>
            <FrameLabel icon="waves">Mobil · PWA (iOS)</FrameLabel>
            <IOSDevice width={402} height={846}>
              <Balager device="mobile" sidebarDefault={t.sidebarOpen} />
            </IOSDevice>
          </div>
        </div>
      </div>

      <TweaksPanel>
        <TweakSection label="Hangulat" />
        <TweakColor label="Víz színe" value={t.water}
          options={Object.values(WATER_PALETTES)}
          onChange={v => setTweak('water', v)} />
        <TweakSection label="Elrendezés" />
        <TweakRadio label="Sűrűség" value={t.density}
          options={['compact', 'regular', 'roomy']}
          onChange={v => setTweak('density', v)} />
        <TweakSlider label="Sarok-lekerekítés" value={t.radius} min={4} max={22} step={1} unit="px"
          onChange={v => setTweak('radius', v)} />
        <TweakToggle label="Nyitott oldalsáv (asztali)" value={t.sidebarOpen}
          onChange={v => setTweak('sidebarOpen', v)} />
      </TweaksPanel>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<Canvas />);
