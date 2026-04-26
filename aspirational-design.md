The key is:

- **plain Maud remains valid**
- but if you want the premium component authoring experience, you opt into it
- and the derive gives you a **builder-centric, zero-regret API**

---

# Vision

```rust
#[derive(mx::Component)]
struct Card<'a> {
    title: &'a str,
    subtitle: Option<&'a str>,
    tone: Tone,
    body: mx::Slot,
    actions: mx::Slot,
}
```

becomes:

- a typed builder
- typed slot setters
- optional/default ergonomics
- repeated/named child ergonomics
- render support
- local CSS/JS composition hooks
- beautiful compile-time diagnostics

The **struct is the truth**.  
The macro turns that truth into the nicest possible component API.
