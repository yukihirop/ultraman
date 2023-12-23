# CHANGELOG

## v0.3.1

- [Breaking] fix `port_for` logic.
  - Ports may be specified explicitly, so I stopped adjusting them automatically with `app_index`.

```rs
// before
pub fn port_for(env_path: &PathBuf, port: Option<u32>, app_index: usize, concurrency_index: usize) -> u32 { 
  base_port(env_path, port) + (app_index * 100 + concurrency_index) as u32 
} 

// after
pub fn port_for(env_path: &PathBuf, port: Option<u32>, concurrency_index: usize) -> u32 { 
  base_port(env_path, port) + concurrency_index
} 
```

## v0.1.2

- Refactor All
  - [Breaking] The output format of `ultraman export <format> <location>` has changed
- Add Badges

Please see [milestone v0.1.2](https://github.com/yukihirop/ultraman/milestone/3?closed=1)

## v0.1.1

2020/12/24

- Support to install by homebrew

Please see [milestone v0.1.1](https://github.com/yukihirop/ultraman/milestone/2?closed=1)

## v0.1.0

2020/12/13

First Release 🎉

[foreman](https://github.com/ddollar/foreman)'s rust implementation

- ultraman start
- ultraman run <command>
- ultraman export <format> <location>

Please see [milestone v0.1.0](https://github.com/yukihirop/ultraman/milestone/1?closed=1)
