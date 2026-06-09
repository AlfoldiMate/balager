# Balager

Balager is a web application to manage the family reservations, discussions, and
todos for our weekend house at Lake Balaton.

> **Language convention:** code, plans, and documentation are written in English.
> The user-facing interface is in Hungarian.

## Features

### Users
- Users cannot register themselves — account creation is an administrator action.
- Two user types: **normal** and **approver**.
- Users can manage their own notification settings (email and PWA).

### Reservations
- Reservations are valid for whole days (start/end times are not tracked).
- Approval status:
  - **Requested**
  - **Approved** — when every approver in the system has approved it.
  - **Rejected** — when any approver rejects it (a comment explaining why is required).
- Visibility / attendance status:
  - **Closed** — only the reserver manages the attendees.
  - **Open** — any registered user can attend.

### Tasks
- Tasks can be grouped.
- Tasks can have subtasks.
- Tasks can be recurring.
- Tasks can have attached events, which result in an **open** reservation.

### Discussions
- Threads can be created for users to discuss different topics.
- Messages can be answered (as a sub-thread), pinned, and up/down-voted.
- Discussions can be closed by approvers.
- Discussions can be attached to tasks or reservations — initiated from the
  reservation or task views, not from the discussions app itself.

## Design

- Inspired by the Balaton feeling and traditional motifs.
- Palette built around water and lake: greens, wood/reed tones, sunny, hot, light.
- Clean and modern.
- Easy to use, logical, intuitive — relevant, context-dependent information
  surfaces when needed.

## UX

- Icon-based side navigation that can be toggled to reveal text labels.
- **Reservations tool**
  - Continuously scrollable view with weeks aligned.
  - Days are colored by status:
    - No reservation — empty
    - Pending — pink / orange-ish (matching the design)
    - Rejected — reddish (matching the design)
    - Accepted — blue-ish for closed, green-ish for open
  - Reserve by selecting unreserved days.
  - Related discussions can be created/seen; status and attendance changes are
    propagated by the system into the discussion.
- **Tasks tool**
  - A typical, intuitive task-management tool per the specification above.
  - Related discussions can be created/seen; status and attendance changes are
    propagated by the system into the discussion.
- **Discussions tool**
  - As specified above.
  - Task-related and reservation-related discussions can be viewed separately.
- **Information tool**
  - Explains the current features, how to use the application, and the rules.

## Technology

- Rust full-stack application: Dioxus 0.7 (WASM client, CSR) + axum API of `#[server]` functions
- PostgreSQL database via sqlx (`DATABASE_URL`; Neon/Vercel Postgres in production)
- Deployed on **Vercel**: static client on the CDN + one Rust Fluid function (`@vercel/rust`) for the API
- Login via email and password (argon2), 180-day session cookie
- Desktop and mobile design, minimal PWA support (for iOS)
- Email notifications via SMTP (optional, see below)
- Styling: the design prototype's CSS ported 1:1 (`assets/styles.css`); Tailwind can be layered in later

See `docs/PLAN.md` for the architecture and domain rules.

## Development

Local development needs a Postgres database, e.g.:

```sh
brew install postgresql@17 && brew services start postgresql@17
createdb balager
export DATABASE_URL=postgres://$USER@localhost:5432/balager
./scripts/dev.sh        # builds the client into ./public, serves on :3000
```

On first run the schema is migrated and one approver account is seeded:
**admin@balager.hu / balaton26** (override via `BALAGER_ADMIN_EMAIL` /
`BALAGER_ADMIN_PASSWORD`). Log in with it, change the password in Beállítások,
and create the family's accounts there (users cannot register themselves).

### Email notifications (optional)

Set these env vars to enable notification emails; without them emails are
skipped silently and only in-app notifications are produced:

```
SMTP_HOST, SMTP_PORT (default 587), SMTP_USERNAME, SMTP_PASSWORD,
SMTP_FROM (e.g. "Balager <balager@example.hu>"), APP_BASE_URL
```

### Deploying to Vercel

1. Create a Postgres database (Vercel dashboard → Storage → Neon) and note its
   connection string.
2. `npm i -g vercel && vercel link` (or import the Git repo in the dashboard).
3. Set environment variables on the project: `DATABASE_URL` (required),
   optionally `BALAGER_ADMIN_EMAIL`, `BALAGER_ADMIN_PASSWORD` and the SMTP vars.
4. `vercel --prod`.

The build runs `scripts/vercel-build.sh` (compiles the WASM client into
`public/`, served by the CDN); `@vercel/rust` compiles `src/main.rs` into a
Fluid function that answers `/api/*`. The same binary also works self-hosted:
`cargo build --release` and run it next to a `public/` directory.

## Design reference

- Local handoff bundle: `design/design-handoff/` (authoritative)
- Original design source:
  https://api.anthropic.com/v1/design/h/NJk_OEr_w5fOevRnqc_8Pg?open_file=Balager.html
