https://github.com/11Takanori/actix-web-clean-architecture-sample

## Todos

- More api tests for [calendarevent, booking]
- reminders with webhook calls
- account with alloweduseractions
- delete user cleanup actions
- refactor event domain (maybe add more rruleoptions validation to rrule.rs)

## Backlog

- smarter mongodb schema
- is more account admin routes needed?
- protect create account with real emails
- frontend for booking

## Need to have a data model that will support google and outlook calendars in the future

- oauth2.0 flow with redirect to our frontend customized with account logo
- How to specify google and outlook calendar ids ?
  - on calendar level you can replicate to a selected google calendar id and outlook calendar id
  - on resource level you can specify google calendar ids and outlook calendar ids
