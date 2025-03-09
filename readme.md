
## clunk

`clunk` is a small library that makes it easy to use lua as a configuration language in
rust projects.

### usage

### development

### limitations

can't have a lua config like:

```lua
c = { 0, 1, 2 }
```

and a struct like:

```rust
struct C(Vec<u8>);
```

### todos

- [ ] improve error messages for nested structs
- [ ] aliases
- [ ] extra examples & tests
- [ ] support for calling rust functions in lua
- [ ] support for calling lua functions in rust?

### why lua?

for personal projects, I've used a lot of various configuration languages/formats including
[ini](https://en.wikipedia.org/wiki/INI_file),
[yaml](https://yaml.org/),
[toml](https://toml.io/),
[json5](https://json5.org/),
[kdl](https://kdl.dev/),
and the list goes on...
there's things I like and dislike about each of them, ranging from tooling support to syntax to 
complexity.
however, I've never used lua-as-configuration before, or any _real_ language to specify
configuration.
this is my attempt to explore and experiment in that space.

[cue](https://cuelang.org/) looks very cool and will probably be the next thing I try :)
