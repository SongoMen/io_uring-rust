# io_uring_rust: Minimal Async Runtime

A minimal implementation of an asynchronous Rust runtime built on top of Linux's **io_uring** interface. 

# Structure:

lib.rs - The main library entry point.

driver.rs - Manages the io_uring instance, the submission queue (SQ), and the completion queue (CQ).

fs.rs - Provides File structs and implements Future for I/O operations

buffer.rs - Defines unsafe traits (IoBuf, IoBufMut) to handle passing ownership of buffers to the kernel
