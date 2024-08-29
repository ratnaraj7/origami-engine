## Project Tasks

### High Priority

- [ ] Implement pattern matching
- [ ] Write initial tests for critical components
- [ ] Use `&mut String` for HTML escaping because [`html-escape::encode_text_to_string`](https://docs.rs/html-escape/0.2.13/html_escape/fn.encode_text_to_string.html) requires it, and use the original string for other operations instead of pointer indirection in `push_str`
- [ ] Consolidate attributes when iterating

### Medium Priority

- [ ] Minify scripts and styles using [`minify-html`](https://crates.io/crates/minify-html)
- [ ] Enable moving scripts and styles to desired positions when used inside components (if possible)
- [ ] Write the README documentation
  - [ ] Create a website in docs workspace based on the README using any markdown to html crates (like [pulldown-cmark](https://crates.io/crates/pulldown-cmark))
  - [ ] Write doctests using [`skeptic`](https://crates.io/crates/skeptic)
- [ ] Add [`tailwind_fuse`](https://crates.io/crates/tailwind_fuse) feature

### Low Priority

- [ ] Write benchmarks using [`criterion`](https://crates.io/crates/criterion) to measure and optimize performance
- [ ] Refactor code for readability and maintainability
- [ ] Document API endpoints
