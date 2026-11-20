//! Runs custom metric plugins: WebAssembly modules exporting an `i32 -> i32`
//! function (for example a scoring function over a commit count). Plugins are
//! sandboxed by wasmtime, so teams can ship metrics without touching the core.
//!
//!   wasm-run module.wat fib 20

use anyhow::{Context, Result, anyhow, bail};
use wasmtime::{Engine, Instance, Module, Store};

fn call(module_bytes: &[u8], func: &str, arg: i32) -> Result<i32> {
    let engine = Engine::default();
    let module =
        Module::new(&engine, module_bytes).map_err(|e| anyhow!("compiling module: {e}"))?;
    let mut store = Store::new(&engine, ());
    let instance =
        Instance::new(&mut store, &module, &[]).map_err(|e| anyhow!("instantiating: {e}"))?;
    let f = instance
        .get_typed_func::<i32, i32>(&mut store, func)
        .map_err(|e| anyhow!("export `{func}` is not an i32 -> i32 function: {e}"))?;
    f.call(&mut store, arg).map_err(|e| anyhow!("trap: {e}"))
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [path, func, arg] = args.as_slice() else {
        bail!("usage: wasm-run <module.wasm|.wat> <export> <i32>");
    };
    let bytes = std::fs::read(path).with_context(|| format!("reading {path}"))?;
    println!(
        "{}",
        call(
            &bytes,
            func,
            arg.parse().context("argument must be an i32")?
        )?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIB: &str = r#"
        (module
          (func $fib (export "fib") (param $n i32) (result i32)
            (if (result i32) (i32.lt_s (local.get $n) (i32.const 2))
              (then (local.get $n))
              (else (i32.add
                (call $fib (i32.sub (local.get $n) (i32.const 1)))
                (call $fib (i32.sub (local.get $n) (i32.const 2))))))))
    "#;

    #[test]
    fn runs_exported_function() {
        assert_eq!(call(FIB.as_bytes(), "fib", 20).unwrap(), 6765);
    }

    #[test]
    fn missing_export_is_an_error() {
        assert!(call(FIB.as_bytes(), "nope", 1).is_err());
    }
}
