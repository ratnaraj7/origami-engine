## Project Tasks

### High Priority

- [ ] Refactor the code to improve logic, optimize performance, and enhance maintainability, as the initial implementation was focused on just making it work.
- [ ] Update attributes handling logic
- [x] Implement pattern matching
- [x] Write initial tests for critical components
- [ ] ~~Use `&mut String` for HTML escaping because [`html-escape::encode_text_to_string`](https://docs.rs/html-escape/0.2.13/html_escape/fn.encode_text_to_string.html) requires it, and use the original string for other operations instead of pointer indirection in `push_str`~~
- [ ] ~~Consolidate attributes when iterating~~
- [ ] For literals escape or minify at compile time

### Medium Priority

- [x] Minify scripts and styles using [`minify-html`](https://crates.io/crates/minify-html)
- [x] Enable moving scripts and styles to desired positions when used inside components (if possible)
- [x] Write the README documentation
  - [ ] ~~Create a website in docs workspace based on the README using any markdown to HTML crates (like [pulldown-cmark](https://crates.io/crates/pulldown-cmark))~~
  - [ ] ~~Write doctests using [`skeptic`](https://crates.io/crates/skeptic)~~
- [ ] ~~Add [`tailwind_fuse`](https://crates.io/crates/tailwind_fuse) feature~~

### Low Priority

- [ ] Write benchmarks using [`criterion`](https://crates.io/crates/criterion) to measure and optimize performance

### Ideas

- [ ] ~~Macro calling another macro for blocks, consolidation, etc.~~
