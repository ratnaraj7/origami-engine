## Project Tasks

### High Priority

- [ ] Implement pattern matching
- [ ] Write initial tests for critical components
- [ ] Use `&mut String` for HTML escaping because `html-escape::encode_text_to_string` requires it, and use the original string for other operations instead of pointer indirection in `push_str`

### Medium Priority

- [ ] Minify scripts and styles using `minify-html`
- [ ] Enable moving scripts and styles to desired positions when used inside components (if possible)
- [ ] Write the README documentation
  - [ ] Create a website in docs workspace based on the README using any markdown to html crates (like [pulldown-cmark](https://crates.io/crates/pulldown-cmark))
  - [ ] Write doctests using `skeptic`

### Low Priority

- [ ] Write benchmarks using `criterion` to measure and optimize performance
- [ ] Refactor code for readability and maintainability
- [ ] Document API endpoints
