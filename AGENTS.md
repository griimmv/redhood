## Compilation Error Workflow

When compilation fails, follow this exact workflow before fixing anything.

### Step 1: Group errors by category

Read the full `cargo check` output and classify each error into one of:

| Category | Meaning |
|---|---|
| **Path / Import** | Module, crate, or symbol not found |
| **Missing Feature** | Cargo feature flag needed |
| **Type Mismatch** | Wrong types used, no implicit coercion |
| **Wrong API** | Method/field doesn't exist on that type |
| **Borrow Checker** | Lifetime, ownership, or `'static` violation |
| **Signature** | Wrong number/type of arguments |
| **Keyword** | Reserved keyword collision (Rust edition) |
| **Async Context** | `.await` outside `async fn`/block |
| **Trait Bound** | Missing `use` import or trait not implemented |

### Step 2: Present grouped errors to the user

Format each group as:

```
### Category Name (N errors)

| Error | Why |
|---|---|
| `error code: message` | Explanation of why Rust rejected it |

**Lesson:** What to do differently next time to avoid this.

---
```

End with a summary:

```
**Total:** N errors across M categories

Fix these? (yes/no)
```

### Step 3: Wait for confirmation

Do **not** fix anything until the user says yes.
