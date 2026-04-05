# clap-tui-macros

`clap-tui-macros` provides the procedural macro support for
[`clap-tui`](https://github.com/gitseti/clap-tui), including the
`#[clap_tui::main]` attribute that wraps a `clap` parser in the
`clap_tui::ParserLauncher` runtime.

Most applications should depend on `clap-tui` directly rather than adding
`clap-tui-macros` as a standalone dependency.

## Example

```rust
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long)]
    name: String,
}

#[clap_tui::main]
fn main(cli: Cli) -> Result<(), clap_tui::TuiError> {
    println!("Hello, {}!", cli.name);
    Ok(())
}
```
