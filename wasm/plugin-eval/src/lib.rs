//! cworks test plugin: an expression evaluation service.
//!
//! Subscribes to `/srv/eval/req`, evaluates each received expression with
//! `evalexpr`, and publishes the result to `/srv/eval/res`.

use std::cell::RefCell;
use std::rc::Rc;

use cworks_sdk::prelude::*;

#[cworks_sdk::plugin]
async fn main(session: Session) {
    let _ = run(session).await;
}

async fn run(mut session: Session) -> Result<(), SyscallError> {
    session.fs_mkdir("/".to_string(), "srv".to_string()).await?;
    session.fs_mkdir("/srv".to_string(), "eval".to_string()).await?;

    let pending: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let pending_handler = pending.clone();
    let handler: DataHandler = Rc::new(Box::new(
        move |data: Option<FSObjRef>| {
            if let Some(obj) = data {
                if let Object::String(s) = &**obj.borrow() {
                    *pending_handler.borrow_mut() = Some(s.clone());
                }
            }
            Ok(())
        },
    ));
    session
        .subscribe("/srv/eval/req".to_string(), handler)
        .await?;

    loop {
        session.wait_for_event().await?;
        if let Some(expr) = pending.borrow_mut().take() {
            let reply = match evalexpr::eval(&expr) {
                Ok(value) => value.to_string(),
                Err(e) => format!("Error: {e}"),
            };
            session
                .publish(
                    "/srv/eval/res".to_string(),
                    Some(Object::String(reply).into()),
                )
                .await?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cworks_sdk::{step_plugin, PollResult, Session, SyscallData};
    use std::future::Future;
    use std::pin::Pin;

    fn drive(
        session: &mut Session,
        fut: &mut Pin<Box<dyn Future<Output = ()>>>,
        req: &SyscallData,
    ) -> PollResult {
        let input = postcard::to_allocvec(req).unwrap();
        let out = step_plugin(session, fut.as_mut(), &input);
        let body = out.get(4..).expect("missing length prefix");
        postcard::from_bytes(body).expect("decode failed")
    }

    #[test]
    fn eval_service_flow() {
        let mut session = Session::default();
        let run_session = session.clone();
        let mut fut: Pin<Box<dyn Future<Output = ()>>> =
            Box::pin(async move { let _ = run(run_session).await; });

        let r = drive(&mut session, &mut fut, &SyscallData::None);
        assert!(
            matches!(r, PollResult::Mkdir(ref p, ref n) if p == "/" && n == "srv"),
            "step1: {r:?}"
        );

        let r = drive(&mut session, &mut fut, &SyscallData::FSSuccess);
        assert!(
            matches!(r, PollResult::Mkdir(ref p, ref n) if p == "/srv" && n == "eval"),
            "step2: {r:?}"
        );

        let r = drive(&mut session, &mut fut, &SyscallData::FSSuccess);
        assert!(
            matches!(r, PollResult::Subscribe(ref p) if p == "/srv/eval/req"),
            "step3: {r:?}"
        );

        let r = drive(&mut session, &mut fut, &SyscallData::FSSuccess);
        assert!(matches!(r, PollResult::WaitForEvent), "step4: {r:?}");

        let invoke = SyscallData::Invoke {
            caller_pid: 9,
            path: "/srv/eval/req".to_string(),
            arg: Some(Object::String("2+3*4".to_string()).into()),
        };
        let r = drive(&mut session, &mut fut, &invoke);
        assert!(
            matches!(r, PollResult::Publish(ref p, Some(ref arg))
                if p == "/srv/eval/res"
                    && matches!(&**arg.borrow(), Object::String(s) if s == "14")),
            "step5: {r:?}"
        );
    }
}
