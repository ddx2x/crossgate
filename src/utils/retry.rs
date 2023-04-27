use std::time::Duration;

use async_recursion::async_recursion;
use futures::future::BoxFuture;
use tokio_context::context::{Context as Ctx, RefContext};

pub type Task = dyn Fn(Ctx) -> BoxFuture<'static, anyhow::Result<()>>;
pub trait Tasker {
    fn run(&self, ctx: Ctx) -> BoxFuture<'static, anyhow::Result<()>>;
}

#[async_recursion(?Send)]
pub async fn retry(ctx: Ctx, t: &Task, jiter: Duration, take: i8) -> anyhow::Result<()> {
    let (_, mut h) = Ctx::with_parent(&RefContext::from(ctx), None);
    if let Err(e) = t(h.spawn_ctx()).await {
        let take = match take {
            0 => return Err(e),
            -1 => -1,
            _ => take - 1,
        };
        tokio::time::sleep(jiter).await;
        retry(h.spawn_ctx(), t, jiter, take).await?;
    }

    Ok(())
}

#[async_recursion(?Send)]
pub async fn retry2(ctx: Ctx, t: Box<dyn Tasker>, jiter: Duration, take: i8) -> anyhow::Result<()> {
    let (_, mut h) = Ctx::with_parent(&RefContext::from(ctx), None);
    if let Err(e) = t.run(h.spawn_ctx()).await {
        let take = match take {
            0 => return Err(e),
            -1 => -1,
            _ => take - 1,
        };
        tokio::time::sleep(jiter).await;
        retry2(h.spawn_ctx(), t, jiter, take).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use futures::future::BoxFuture;
    use std::time::Duration;
    use tokio_context::context::Context;

    fn foo(ctx: Context) -> BoxFuture<'static, anyhow::Result<()>> {
        Box::pin(async move {
            println!("foo");
            Err(anyhow::anyhow!("foo error"))
        })
    }

    struct Foo;

    impl Tasker for Foo {
        fn run(&self, ctx: Context) -> BoxFuture<'static, anyhow::Result<()>> {
            Box::pin(async move {
                println!("foo");
                Err(anyhow::anyhow!("foo error"))
            })
        }
    }

    #[tokio::test]
    async fn test_retry() {
        let (ctx, h) = Context::new();

        tokio::select! {
            Err(e) = retry(ctx, &foo, Duration::from_secs(3), -1) => {
                println!("error: {}", e);
            },
            _ = tokio::signal::ctrl_c() => {
                h.cancel();
                println!("ctrl-c");
            }
        }
    }

    #[tokio::test]
    async fn test_retry2() {
        let (ctx, h) = Context::new();

        tokio::select! {
            Err(e) = retry2(ctx, Box::new(Foo{}), Duration::from_secs(3), -1) => {
                println!("error: {}", e);
            },
            _ = tokio::signal::ctrl_c() => {
                h.cancel();
                println!("ctrl-c");
            }
        }
    }
}
