---
name: rust-concurrency
description: Guides concurrency and async decisions in Rust. Use when encountering Send/Sync errors (E0277), choosing between threads vs async, designing shared state, or debugging deadlocks.
license: MIT
origin_url: https://github.com/actionbook/rust-skills/tree/main/skills
---

# Concurrency

> **Layer 1: Language Mechanics**

## Core Question

**Is this CPU-bound or I/O-bound, and what's the sharing model?**

Before choosing concurrency primitives:
- What's the workload type?
- What data needs to be shared?
- What's the thread safety requirement?

---

## Thinking Prompt

Before adding concurrency:

1. **What's the workload?**
   - CPU-bound → threads (`std::thread`, rayon)
   - I/O-bound → async (tokio, async-std)
   - Mixed → hybrid approach

2. **What's the sharing model?**
   - No sharing → message passing (channels)
   - Immutable sharing → `Arc<T>` when `T: Send + Sync`
   - Mutable sharing → `Arc<Mutex<T>>` or `Arc<RwLock<T>>` when the inner type and lock choice satisfy the required bounds

3. **What are the Send/Sync requirements?**
   - Cross-thread ownership → `Send`
   - Cross-thread references → `Sync`
   - Single-thread async → `spawn_local` only on a compatible local executor (for example, Tokio `LocalSet` or current-thread runtime)

---

## Decision Flowchart

```
What type of work?
├─ CPU-bound → std::thread or rayon
├─ I/O-bound → async/await
└─ Mixed → hybrid (spawn_blocking)

Need to share data?
├─ No → message passing (channels)
├─ Immutable → Arc<T>
└─ Mutable →
   ├─ Read-heavy → Arc<RwLock<T>>
   └─ Write-heavy → Arc<Mutex<T>>
   └─ Simple counter → AtomicUsize

Async context?
├─ Future is Send → tokio::spawn (on a Tokio runtime)
├─ Future is !Send → spawn_local only with a local executor/runtime context
└─ Blocking code → the runtime's blocking facility (for example, tokio::task::spawn_blocking)
```

---

## Send/Sync Markers

| Marker | Meaning | Example |
|--------|---------|---------|
| `Send` | Can transfer ownership between threads | Many types; depends on all contained fields |
| `Sync` | Can share references between threads | Types whose shared references are safe; depends on all contained fields |
| `!Send` | Must stay on one thread | `Rc<T>` |
| `!Sync` | No shared refs across threads | `RefCell<T>` |

## Quick Reference

| Pattern | Thread-Safe | Blocking | Use When |
|---------|-------------|----------|----------|
| `std::thread` | Depends on captured values being `Send` | Yes | CPU-bound parallelism |
| `async/await` | A future is not necessarily `Send` | Cooperatively scheduled; avoid blocking the executor | I/O-bound concurrency |
| `Mutex<T>` | Depends on `T` and the mutex implementation | Yes | Shared mutable state |
| `RwLock<T>` | Depends on `T` and the lock implementation | Yes | Read-heavy shared state |
| `mpsc::channel` | Depends on the channel implementation and message type | Optional | Message passing |
| `Arc<Mutex<T>>` | Depends on `T: Send` and the chosen mutex | Yes | Shared mutable across threads |

---

## Async-Specific Patterns

### Avoid MutexGuard Across Await

```rust
// Bad: guard held across await
let guard = mutex.lock().await;
do_async().await;  // guard still held!

// Good: scope the lock
{
    let guard = mutex.lock().await;
    // use guard
}  // guard dropped
do_async().await;
```

### Non-Send Types in Async

```rust
// Rc is !Send, can't cross await in spawned task
// Option 1: use Arc instead
// Option 2: use spawn_local (single-thread runtime)
// Option 3: ensure Rc is dropped before .await
```

---

## Error → Design Question

| Error | Don't Just Say | Ask Instead |
|-------|----------------|-------------|
| E0277 Send | "Add Send bound" | Should this type cross threads? |
| E0277 Sync | "Wrap in Mutex" | Is shared access really needed? |
| Future not Send | "Use spawn_local" | Is async the right choice? |
| Deadlock | "Reorder locks" | Is the locking design correct? |

---

## Anti-Patterns

| Anti-Pattern | Why Bad | Better |
|--------------|---------|--------|
| `Arc<Mutex<T>>` everywhere | Contention, complexity | Message passing |
| `thread::sleep` in async | Blocks executor | `tokio::time::sleep` |
| Holding locks across await | Blocks other tasks | Scope locks tightly |
| Ignoring deadlock risk | Hard to debug | Lock ordering, `try_lock` |

---

## Related Skills

- `ownership` — data ownership and borrowing
- `performance` — choosing between rayon, threads, and async for throughput
- `anti-patterns` — common concurrency mistakes
