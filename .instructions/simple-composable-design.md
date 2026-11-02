# Simple, composable design

Avoid ceremony. Prefer features that emerge naturally from a simple, composable
design rather than being tacked on to match the wording of a request. Identify
the underlying need and choose primitives that address it cleanly. Derive
information already available instead of requiring redundant declarations or
bookkeeping.

Separate reusable algorithms from application policy and integration. Favor small,
cohesive libraries with tight public interfaces that are easy to explain. Use
standard interfaces and typed context parameters where they fit. Add abstractions
when they clarify a real mechanism.
