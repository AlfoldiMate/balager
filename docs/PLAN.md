# Balager — Fullstack Architecture Plan

Goal: turn the design-faithful UI into a real application the family can use:
login, persistent data, working reservations/tasks/discussions, notifications.

## Stack

| Layer | Choice | Rationale |
|---|---|---|
| Frontend | Dioxus 0.7 (WASM, client-side rendered) | Spec: Rust + Dioxus |
| API | Dioxus `#[server]` functions over axum, one Vercel Fluid function | One codebase, typed end-to-end |
| Database | PostgreSQL via sqlx (Neon / Vercel Postgres) | Serverless has no disk; Neon free tier fits family scale |
| Auth | email+password, argon2 hashes, DB session token in HttpOnly cookie (180 days) | Spec: login + long-remaining auth |
| Email | lettre over SMTP, configured via env; silently skipped when unconfigured | Spec: email notifications |
| Hosting | Vercel: static client on CDN + `@vercel/rust` function for `/api/*` | Spec: Vercel deployment; same binary self-hosts |
| Styling | The design prototype's CSS ported 1:1 (`assets/styles.css`) | Pixel fidelity to the approved design; Tailwind can be layered in later for new screens |

### Deployment

`scripts/vercel-build.sh` compiles the WASM client into `public/` (served by
the Vercel CDN); `@vercel/rust` compiles `src/main.rs` (bin `main`, default
features) into a Fluid function answering `/api/*` (`vercel.json` rewrites).
Locally and self-hosted the very same binary is a normal HTTP server on :3000
that also serves `./public`. Requires `DATABASE_URL`.

## Module layout & layering

```
src/
  main.rs          bin "main": Vercel function / local HTTP server; web: launch
  models.rs        shared serde DTOs + Hungarian date/label helpers (chrono)
  api/             TRANSPORT: #[server] endpoints — authenticate, delegate, map errors
    auth.rs users.rs reservations.rs tasks.rs discussions.rs notifications.rs
  domain/          DOMAIN: all business rules, transport-agnostic (server-only)
    mod.rs         DomainError, Actor, authorization guards
    users.rs reservations.rs tasks.rs discussions.rs notifications.rs
  backend/         INFRASTRUCTURE: DB pool/migrations/seed, sessions/passwords,
    db.rs auth.rs notify.rs        notification fan-out + SMTP delivery
  app.rs state.rs login.rs shell.rs reservations.rs tasks.rs discussions.rs
  info.rs notifications.rs settings.rs common.rs icons.rs   (views, web-only)
```

The dependency direction is `api → domain → backend`; views talk only to
`api` stubs and `models`. Authentication (cookie → `Actor`) happens at the
transport edge; authorization (approver checks, ownership) lives in the domain
so every caller gets the same rules.

## Extensibility

Adding a feature touches the layers top-down and nothing else:

1. **New rule on an existing operation** — edit the one domain function; every
   transport (web today, anything tomorrow) picks it up.
2. **New operation** — add a `domain::<area>` function (rules + notification
   fan-out), a 5-line `#[server]` endpoint, and the UI that calls it.
3. **New entity** — migration in `migrations/`, DTO in `models.rs`, domain
   module, endpoints, view.
4. **New entry point** — the domain layer has no HTTP types in its signatures
   (`Actor` + plain data in, `DomainResult` out), so a cron binary for
   due-date reminders, an admin CLI, or a Telegram bot can call it directly;
   only session/cookie handling is web-specific.
5. **Swapping infrastructure** — email delivery is isolated in
   `backend/notify.rs` (e.g. switch SMTP → Resend API in one file); the DB is
   plain sqlx/Postgres behind `backend/db.rs`.

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
