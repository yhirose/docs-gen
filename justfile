default: list

list:
    @just --list --unsorted

clean:
    @rm -rf docs*

install:
    @cargo install --path .

test:
    @cargo test --release
    @cargo test --release --test e2e -- --ignored --test-threads=1

init:
    @docs-gen init docs-src

build theme="default":
    @docs-gen build docs-src docs --theme={{theme}}

serve theme="default":
    @docs-gen serve docs-src --theme={{theme}} --open

check:
    @docs-gen check docs-src

release version:
    @scripts/release {{version}}
