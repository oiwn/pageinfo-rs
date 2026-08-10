# Current Task Context: Remove Obscura rendering
State: in progress

## Plan

- [x] Record future browser-rendering directions in the specs
- [x] Remove the Obscura dependency and rendering interfaces
- [x] Restore crates.io publishing and HTTP-only documentation
- [x] Verify the default build and publish package

## Context

Rendering is not currently required. Remove Obscura now; Chromium/CDP is
committed future work, while Servo remains a far-future idea.

## Next

Await explicit confirmation before archiving this completed task.
