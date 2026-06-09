// notifications.jsx — bell popover (desktop) / full sheet (mobile)
function NotifList({ onClose }) {
  const unread = NOTIFS.filter(n => n.unread).length;
  return (
    <React.Fragment>
      <div className="nh">
        <h3>Értesítések</h3>
        {unread > 0 && <span className="npill">{unread} új</span>}
        <button className="bg-iconbtn" style={{ width: 32, height: 32, border: 'none', background: 'transparent' }} onClick={onClose}><Icon name="x" size={17} /></button>
      </div>
      <div className="nlist">
        {NOTIFS.map(n => (
          <div key={n.id} className={'notif-row' + (n.unread ? ' unread' : '')}>
            <div className={'notif-ic ' + n.tone}><Icon name={n.icon} size={18} stroke={2.2} /></div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div className="ntext">
                {n.who !== 'system' && <b>{USERS[n.who].name} </b>}{n.text}
              </div>
              <div className="ntime">{n.time}</div>
            </div>
            {n.unread && <span className="unreaddot" />}
          </div>
        ))}
      </div>
      <div className="nfoot"><button>Összes megjelölése olvasottként</button></div>
    </React.Fragment>
  );
}

function NotificationsPopover({ onClose }) {
  return (
    <React.Fragment>
      <div className="bg-notif-scrim" onClick={onClose} />
      <div className="bg-notif"><NotifList onClose={onClose} /></div>
    </React.Fragment>
  );
}

window.NotificationsPopover = NotificationsPopover;
window.NotifList = NotifList;
