# Balager — Fullstack Architecture Plan

Goal: turn the design-faithful UI into a real application the family can use:
login, persistent data, working reservations/tasks/discussions, notifications.

## Stack

| Layer | Choice | Rationale |
|---|---|---|
| Frontend | Dioxus 0.7 (WASM, SSR + hydration) | Spec: Rust + Dioxus |
| API | Dioxus `#[server]` functions over axum | One codebase, typed end-to-end |
| Database | SQLite via sqlx (async) | Family-scale data, zero ops; file DB, easy backup |
| Auth | email+password, argon2 hashes, DB session token in HttpOnly cookie (180 days) | Spec: login + long-remaining auth |
| Email | lettre over SMTP, configured via env; silently skipped when unconfigured | Spec: email notifications |
| Styling | The design prototype's CSS ported 1:1 (`assets/styles.css`) | Pixel fidelity to the approved design; Tailwind can be layered in later for new screens |

### Deployment note

`dx bundle --platform web` produces a single self-contained axum binary +
static assets. Vercel does not host long-running Rust servers, so the practical
targets are Fly.io / Hetzner / any VPS / a Raspberry Pi at home. The SQLite
file lives next to the binary (`DATABASE_URL`, default `sqlite:balager.db`).

## Module layout

```
src/
  main.rs          server: dioxus::serve (DB init → router); client: launch
  app.rs           root: session gate → Login | Shell
  models.rs        shared serde DTOs + Hungarian date/label helpers (chrono)
  state.rs         UI state + data resources (me, users, reservations, groups, threads, notifs)
  api/             #[server] functions = the application/domain layer
    auth.rs users.rs reservations.rs tasks.rs discussions.rs notifications.rs
  backend/         server-only (cfg feature = "server")
    db.rs          pool, migrations (./migrations), first-run seed
    auth.rs        password hashing, session create/validate, cookie helpers
    notify.rs      notification fan-out + email sending honoring prefs
  login.rs shell.rs reservations.rs tasks.rs discussions.rs info.rs
  notifications.rs settings.rs common.rs icons.rs   (views)
```

## Domain rules (enforced server-side)

- **Users**: no self-registration. Approvers administer users (create,
  activate/deactivate, reset password, role). Roles: `normal`, `approver`.
  First run seeds one approver (`BALAGER_ADMIN_EMAIL`/`BALAGER_ADMIN_PASSWORD`,
  defaults `admin@balager.hu` / `balaton26` — change after first login).
- **Reservations**: whole days only (`from..=to` ISO dates). On request, one
  approval row per active approver is created. Status is *derived*:
  any `rejected` ⇒ **rejected** (comment mandatory); all `approved` ⇒
  **approved** (shown as *closed*/*open* by access); else **pending**.
  Overlap with any non-rejected reservation is refused. Open: anyone can
  join/leave. Closed: only the owner manages attendees. Owner can switch
  access and delete the reservation.
- **Tasks**: groups → tasks → subtasks. Completing a recurring task advances
  its due date by the interval instead of staying done. "Attach event" creates
  an **open** reservation linked to the task (goes through the normal approval
  flow).
- **Discussions**: threads are `general`, or linked to a reservation/task
  (created from there, not from the discussions tool). Messages support one
  level of replies (sub-thread), pin/unpin, up/down votes (one vote per user),
  and polls (date/list, single/multi choice). Only approvers close/reopen
  threads; author or approver may delete. Status, attendance and decision
  changes are propagated into the linked thread as system messages.
- **Notifications**: every relevant event writes an in-app notification for
  the affected users and sends email if that user's preference allows it.
  Preference keys: res_decision, res_request (approvers), res_join,
  task_assigned, task_due, disc_reply, disc_new. Unread counts drive the bell
  dot; sidebar badges are live (pending reservations / my open tasks / open
  threads).

## Schema (migrations/0001_init.sql)

users, sessions, reservations, reservation_approvals, reservation_attendees,
task_groups, tasks, subtasks, threads, messages, message_votes, polls,
poll_options, poll_votes, notifications, notif_prefs.

## Out of scope for this iteration

- Web push delivery (the e-mail/push preference matrix is persisted; push
  delivery needs VAPID keys + service worker — planned next).
- Mention notifications, due-date reminder scheduler (needs a cron loop).
- Tailwind conversion of the ported design CSS.
