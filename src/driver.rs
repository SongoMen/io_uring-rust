// src/driver.rs
use io_uring::{squeue, IoUring};
use slab::Slab;
use std::io;
use std::task::{Context, Poll, Waker};
use std::cell::RefCell;
use std::rc::Rc;

struct OpState {
    waker: Option<Waker>,
    _resources: Option<Box<dyn std::any::Any>>, 
    result: Option<io::Result<i32>>,
}

pub struct Driver {
    ring: IoUring,
    ops: Slab<OpState>,
}

impl Driver {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            ring: IoUring::new(256)?,
            ops: Slab::with_capacity(64),
        })
    }

    pub fn submit(&mut self, sqe: squeue::Entry, resources: Option<Box<dyn std::any::Any>>) -> u64 {
        let entry = self.ops.vacant_entry();
        let key = entry.key();
        
        entry.insert(OpState {
            waker: None,
            _resources: resources,
            result: None,
        });

        let sqe = sqe.user_data(key as u64);

        unsafe {
            self.ring.submission().push(&sqe).expect("Submission queue full");
        }

        self.ring.submit().expect("Failed to submit");
        
        key as u64
    }

    pub fn wait(&mut self) -> io::Result<()> {
        self.ring.submit_and_wait(1)?;

        loop {
            match self.ring.completion().next() {
                Some(cqe) => {
                    let user_data = cqe.user_data() as usize;
                    let res = cqe.result();

                    if let Some(op) = self.ops.get_mut(user_data) {
                        op.result = Some(if res < 0 {
                            Err(io::Error::from_raw_os_error(-res))
                        } else {
                            Ok(res)
                        });
                        
                        if let Some(waker) = op.waker.take() {
                            waker.wake();
                        }
                    }
                }
                None => break,
            }
        }
        Ok(())
    }

    pub fn poll_op(&mut self, key: u64, cx: &mut Context<'_>) -> Poll<(io::Result<i32>, Option<Box<dyn std::any::Any>>)> {
        let key = key as usize;
        let op = self.ops.get_mut(key).expect("Invalid key");

        if let Some(result) = op.result.take() {
            let resources = op._resources.take();
            self.ops.remove(key);
            Poll::Ready((result, resources))
        } else {
            op.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

thread_local! {
    pub static CURRENT: Rc<RefCell<Driver>> = Rc::new(RefCell::new(Driver::new().unwrap()));
}
