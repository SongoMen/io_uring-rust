use crate::driver::CURRENT;
use crate::buffer::IoBufMut;
use io_uring::{opcode, types};
use std::future::Future;
use std::os::unix::io::RawFd;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io;

pub struct File {
    pub fd: RawFd,
}

impl File {
    pub fn read_at<B: IoBufMut + 'static>(&self, buf: B, offset: u64) -> ReadFuture<B> {
        ReadFuture {
            fd: self.fd,
            buf: Some(buf),
            offset,
            key: None,
        }
    }
}

pub struct ReadFuture<B> {
    fd: RawFd,
    buf: Option<B>,
    offset: u64,
    key: Option<u64>,
}

impl<B: IoBufMut + Unpin + 'static> Future for ReadFuture<B> {
    type Output = (io::Result<usize>, B);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if this.key.is_none() {
            let mut buf = this.buf.take().unwrap();
            let sqe = opcode::Read::new(
                types::Fd(this.fd), 
                buf.stable_mut_ptr(), 
                buf.bytes_init() as _
            )
            .offset(this.offset)
            .build();

            let key = CURRENT.with(|driver| {
                driver.borrow_mut().submit(sqe, Some(Box::new(buf)))
            });
            
            this.key = Some(key);
        }

        let key = this.key.unwrap();
        let poll_result = CURRENT.with(|driver| {
            driver.borrow_mut().poll_op(key, cx)
        });

        match poll_result {
            Poll::Ready((res, resources)) => {
                let buf = *resources.unwrap().downcast::<B>().unwrap();
                let n = res.unwrap_or(0);
                Poll::Ready((Ok(n as usize), buf))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
