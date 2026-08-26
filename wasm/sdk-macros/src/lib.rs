use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Declare a cworks plugin entry point.
///
/// Generates the wasm exports (`cworks_alloc` / `cworks_plugin_poll`)
/// that the host uses to drive the plugin's poll-based state machine.
///
/// ```ignore
/// use cworks_sdk::prelude::*;
///
/// #[cworks_sdk::plugin]
/// async fn main(mut session: Session) {
///     session.subscribe("/srv/eval/req".to_string(), handler).await?;
///     loop { session.wait_for_event().await?; }
/// }
/// ```
#[proc_macro_attribute]
pub fn plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    let expanded = quote! {
        #input_fn

        mod __cworks_plugin {
            use std::cell::RefCell;
            use std::future::Future;
            use std::pin::Pin;

            thread_local! {
                static SESSION: RefCell<Option<cworks_sdk::Session>> =
                    const { RefCell::new(None) };
                static FUTURE: RefCell<Option<Pin<Box<dyn Future<Output = ()>>>>> =
                    const { RefCell::new(None) };
                static INPUT: RefCell<Vec<u8>> =
                    const { RefCell::new(Vec::new()) };
                static OUTPUT: RefCell<Vec<u8>> =
                    const { RefCell::new(Vec::new()) };
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn cworks_alloc(len: usize) -> *mut u8 {
                INPUT.with_borrow_mut(|buf| {
                    buf.clear();
                    buf.resize(len, 0);
                    buf.as_mut_ptr()
                })
            }

            #[unsafe(no_mangle)]
            pub extern "C" fn cworks_plugin_poll(input: *const u8, input_len: usize) -> *const u8 {
                FUTURE.with_borrow_mut(|slot| {
                    if slot.is_none() {
                        let session = <cworks_sdk::Session as Default>::default();
                        SESSION.with_borrow_mut(|s| *s = Some(session.clone()));
                        *slot = Some(Box::pin(super::#fn_name(session)));
                    }

                    let input = unsafe { std::slice::from_raw_parts(input, input_len) };
                    let response = SESSION.with_borrow_mut(|s| {
                        cworks_sdk::step_plugin(
                            s.as_mut().expect("plugin session missing"),
                            slot.as_mut().expect("plugin future missing").as_mut(),
                            input,
                        )
                    });

                    OUTPUT.with_borrow_mut(|buf| {
                        buf.clear();
                        buf.extend_from_slice(&response);
                        buf.as_ptr()
                    })
                })
            }
        }
    };

    expanded.into()
}
