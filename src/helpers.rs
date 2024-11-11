use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::default::Default;


#[derive(Default)]
struct Yield {
    flag: bool,
}

impl Future for Yield {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.flag {
            return Poll::Ready(());
        }

        self.get_mut().flag = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

pub async fn yield_me() {
    Yield::default().await;
}
