#![cfg(any(feature = "with-unix-sockets", not(feature = "with-system-unix-sockets")))]
extern crate redis;

extern crate futures;
extern crate tokio_core;

use futures::Future;
use tokio_core::reactor::Core;

use support::*;

mod support;

#[test]
fn test_args() {
    let mut core = Core::new().unwrap();
    let ctx = TestContext::new();
    let connect = ctx.async_connection(&core.handle());
    let con = core.run(connect).unwrap();

    let (con, ()) = core.run(
        redis::cmd("SET")
            .arg("key1")
            .arg(b"foo")
            .query_async(con)
            .and_then(|(con, ())| redis::cmd("SET").arg(&["key2", "bar"]).query_async(con)),
    ).unwrap();

    assert_eq!(
        core.run(redis::cmd("MGET").arg(&["key1", "key2"]).query_async(con))
            .map(|t| t.1),
        Ok(("foo".to_string(), b"bar".to_vec()))
    );
}

#[test]
fn test_pipeline_transaction() {
    let mut core = Core::new().unwrap();
    let ctx = TestContext::new();
    let connect = ctx.async_connection(&core.handle());
    let con = core.run(connect).unwrap();

    let ((k1, k2),): ((i32, i32),) = core.run(
        redis::pipe()
            .atomic()
            .cmd("SET")
            .arg("key_1")
            .arg(42)
            .ignore()
            .cmd("SET")
            .arg("key_2")
            .arg(43)
            .ignore()
            .cmd("MGET")
            .arg(&["key_1", "key_2"])
            .query_async(&con),
    ).unwrap();

    assert_eq!(k1, 42);
    assert_eq!(k2, 43);
}
