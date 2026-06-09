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

- Rust full-stack application
- Database
- Dioxus framework
- Tailwind CSS
- Planned deployment on Vercel
- Login via email and password
- Long-lived ("remember me") authentication
- Desktop and mobile design, minimal PWA support (for iOS)
- Email notifications about important events

## Design reference

- `Balager.html` — design source:
  https://api.anthropic.com/v1/design/h/NJk_OEr_w5fOevRnqc_8Pg?open_file=Balager.html
