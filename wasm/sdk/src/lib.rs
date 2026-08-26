//! Plugin Development Kit for writing cworks processes in Rust.
//!
//! A plugin is a self-contained wasm module that talks to the cworks kernel
//! through the poll protocol: the host feeds one [`SyscallData`] per call and
//! the plugin answers with the next [`PollResult`], exactly like any other
//! process.
//!
//! The session handle is the kernel's session type (re-exported as
//! [`Session`]), so plugin code reads exactly like native RustProcess code.
//! Channel events are delivered via callback handlers registered with
//! [`Session::subscribe`].
//!
//! ```ignore
//! use cworks_sdk::prelude::*;
//!
//! #[cworks_sdk::plugin]
//! async fn main(mut session: Session) {
//!     let handler: DataHandler = Rc::new(Box::new(|_| Ok(())));
//!     session.subscribe("/srv/eval/req".to_string(), handler).await?;
//!     loop {
//!         session.wait_for_event().await?;
//!     }
//! }
//! ```

use std::future::Future;
use std::rc::Rc;

pub use kernel::obj_tree::{FSObjRef, Object};
pub use kernel::{PollResult, ProcessClientExt, SyscallData, SyscallError};

/// The plugin attribute macro.
pub use cworks_sdk_macros::plugin;

/// The plugin's handle to the kernel — the kernel's session type.
pub use kernel::RustProcessCore as Session;

/// Channel event handler, mirroring the kernel's callback model.
pub type DataHandler = Rc<Box<dyn Fn(Option<FSObjRef>) -> Result<(), SyscallError>>>;

/// Common imports for plugin applications.
pub mod prelude {
    pub use super::{DataHandler, FSObjRef, Object, ProcessClientExt, Session, SyscallError};
}



/// Encode a kernel response as postcard for the host (u32-LE length prefixed).
#[must_use]
pub fn encode_response(res: &PollResult) -> Vec<u8> {
    let mut body = postcard::to_allocvec(res).unwrap_or_default();
    let mut buf = (body.len() as u32).to_le_bytes().to_vec();
    buf.append(&mut body);
    buf
}

/// Decode a host request from postcard (falling back to `None`).
#[must_use]
pub fn decode_request(input: &[u8]) -> SyscallData {
    postcard::from_bytes(input).unwrap_or(SyscallData::None)
}

/// Encode a [`SyscallData`] request as postcard for the plugin.
#[must_use]
pub fn encode_request(data: &SyscallData) -> Vec<u8> {
    postcard::to_allocvec(data).unwrap_or_default()
}

/// Decode a plugin response body from postcard.
#[must_use]
pub fn decode_response(body: &[u8]) -> PollResult {
    postcard::from_bytes(body).unwrap_or(PollResult::Done)
}

/// Host-facing driver: feed one [`SyscallData`], get the plugin's next
/// [`PollResult`], postcard-encoded with a u32-LE length prefix.
#[must_use]
pub fn step_plugin(
    session: &mut Session,
    mut future: std::pin::Pin<&mut dyn Future<Output = ()>>,
    input: &[u8],
) -> Vec<u8> {
    let data = decode_request(input);
    session.pass_syscall_data(&data);

    let waker = std::task::Waker::noop();
    let mut cx = std::task::Context::from_waker(waker);
    let response = if future.as_mut().poll(&mut cx).is_ready() {
        // The entry point completed: terminate the process cleanly.
        PollResult::Done
    } else {
        session.take_syscall()
    };

    encode_response(&response)
}
