// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::shared::{IncomingAuthReq, SharedAuthReqsHandle, SharedSafeAuthenticatorHandle};
use log::{error, info};
use serde_json::{json, Value};
use sn_api::{AuthReq, SafeAuthReq};
use std::time::SystemTime;
use tokio::sync::mpsc;

// Authorisation requests wil be automatically rejected if the number of pending auth reqs reaches this number
// This should never happen and it's just for the containment to keep authd healthy in such an unexpected scenario
const MAX_NUMBER_QUEUED_AUTH_REQS: usize = 64;

enum AuthorisationResponse {
    Ready(Value),
    NotReady((mpsc::Receiver<bool>, u32, String)),
}

pub async fn process_req(
    params: Value,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
) -> Result<Value, String> {
    match handle_authorisation(params, safe_auth_handle.clone(), auth_reqs_handle.clone()).await? {
        AuthorisationResponse::NotReady((rx, req_id, auth_req_str)) => {
            // Let's await for the decision response
            await_authorisation_decision(
                safe_auth_handle,
                auth_reqs_handle,
                rx,
                req_id,
                auth_req_str,
            )
            .await
        }
        AuthorisationResponse::Ready(resp) => Ok(resp),
    }
}

async fn handle_authorisation(
    params: Value,
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
) -> Result<AuthorisationResponse, String> {
    if let Value::String(auth_req_str) = params {
        info!("Authorising application...");
        let safe_authenticator = safe_auth_handle.lock().await;
        match safe_authenticator.decode_req(&auth_req_str).await {
            Ok(SafeAuthReq::Auth(app_auth_req)) => {
                info!(
                    "The following application authorisation request '{}' was received:",
                    app_auth_req.req_id
                );
                info!("{:?}", app_auth_req);

                let mut auth_reqs_list = auth_reqs_handle.lock().await;

                // Reject if there are too many pending auth reqs
                if auth_reqs_list.len() >= MAX_NUMBER_QUEUED_AUTH_REQS {
                    Err(format!("Authorisation request '{}' is rejected by authd since it reached its maximum number ({}) of pending auth requests", app_auth_req.req_id, MAX_NUMBER_QUEUED_AUTH_REQS))
                } else {
                    // We need a channel to communicate with the thread which will be
                    // sending the notification to a subcribed endpoint. Once it got a response
                    // it will send it back through this channel so it can in turn be
                    // sent to the application requesting this authorisation.
                    let (tx, rx): (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel(32);

                    // Let's add it to the list of pending authorisation requests
                    let auth_req = IncomingAuthReq {
                        timestamp: SystemTime::now(),
                        auth_req: AuthReq {
                            req_id: app_auth_req.req_id,
                            app_id: app_auth_req.app_id.clone(),
                            app_name: app_auth_req.app_name.clone(),
                            app_vendor: app_auth_req.app_vendor.clone(),
                        },
                        tx,
                        notified: false,
                    };
                    auth_reqs_list.insert(app_auth_req.req_id, auth_req);

                    Ok(AuthorisationResponse::NotReady((
                        rx,
                        app_auth_req.req_id,
                        auth_req_str.to_string(),
                    )))
                }
            }
            Ok(SafeAuthReq::Unregistered(_)) => {
                // We simply allow unregistered authorisation requests
                match safe_authenticator.authorise_app(&auth_req_str).await {
                    Ok(resp) => {
                        info!("Unregistered authorisation request was allowed and response sent back to the application");
                        Ok(AuthorisationResponse::Ready(json!(resp)))
                    }
                    Err(err) => {
                        error!("Failed to authorise application: {}", err);
                        Err(err.to_string())
                    }
                }
            }
            Err(err) => {
                error!("{}", err);
                Err(err.to_string())
            }
        }
    } else {
        Err(format!(
            "Incorrect params for 'authorise' method: {:?}",
            params
        ))
    }
}

async fn await_authorisation_decision(
    safe_auth_handle: SharedSafeAuthenticatorHandle,
    auth_reqs_handle: SharedAuthReqsHandle,
    mut rx: mpsc::Receiver<bool>,
    req_id: u32,
    auth_req_str: String,
) -> Result<Value, String> {
    match rx.recv().await {
        Some(true) => {
            info!(
                "Let's request the authenticator lib to authorise the auth request with id '{}'...",
                req_id
            );
            let safe_authenticator = safe_auth_handle.lock().await;
            match safe_authenticator.authorise_app(&auth_req_str).await {
                Ok(resp) => {
                    info!("Authorisation request '{}' was allowed and response sent back to the application", req_id);
                    Ok(serde_json::value::Value::String(resp))
                }
                Err(err) => {
                    error!("Failed to authorise application: {}", err);
                    Err(err.to_string())
                }
            }
        }
        Some(false) => {
            let msg = format!("Authorisation request '{}' was denied", req_id);
            info!("{}", msg);
            Err(msg)
        }
        None => {
            // We didn't get a response in a timely manner, we cannot allow the list
            // to grow infinitelly, so let's remove the request from it,
            // even that the notifs thread may have removed it already
            let mut auth_reqs_list = auth_reqs_handle.lock().await;
            auth_reqs_list.remove(&req_id);
            Err("Failed to get authorisation response".to_string())
        }
    }
}
