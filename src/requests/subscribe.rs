// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::shared::SharedNotifEndpointsHandle;
use log::info;
use serde_json::{json, Value};

// Maximum number of allowed auth reqs notifs subscribers
const MAX_NUMBER_OF_NOTIF_SUBSCRIPTIONS: usize = 3;

pub async fn process_req(
    params: Value,
    notif_endpoints_handle: SharedNotifEndpointsHandle,
) -> Result<Value, String> {
    if let Value::Array(args) = &params {
        if args.is_empty() || args.len() > 2 {
            Err(format!(
                "Incorrect number of params for 'subscribe' method: {:?}",
                params
            ))
        } else {
            let mut notif_endpoint = match args[0].as_str() {
                None => return Err(format!(
                    "Invalid endpoint URL string passed as first param for 'subscribe' method: {:?}",
                    params
                )),
                Some(str) => {
                    str.to_string()
                }
            };

            info!("Subscribing to authorisation requests notifications...");
            let cert_base_path = if args.len() == 2 {
                match args[1].as_str() {
                    None => return Err(format!(
                        "Invalid certificate base path string passed as second param for 'subscribe' method: {:?}",
                        params
                    )),
                    Some(str) => {
                        str.to_string()
                    }
                }
            } else {
                "".to_string()
            };

            let mut notif_endpoints_list = notif_endpoints_handle.lock().await;
            // let's normailse the endpoint URL
            if notif_endpoint.ends_with('/') {
                notif_endpoint.pop();
            }

            if notif_endpoints_list.get(&notif_endpoint).is_some() {
                let msg = format!(
                    "Subscription rejected. Endpoint '{}' is already subscribed",
                    notif_endpoint
                );
                info!("{}", msg);
                Err(msg)
            } else if notif_endpoints_list.len() >= MAX_NUMBER_OF_NOTIF_SUBSCRIPTIONS {
                let msg = format!("Subscription rejected. Maximum number of subscriptions ({}) has been already reached", MAX_NUMBER_OF_NOTIF_SUBSCRIPTIONS);
                info!("{}", msg);
                Err(msg)
            } else {
                notif_endpoints_list.insert(notif_endpoint.clone(), cert_base_path.clone());

                let msg = format!(
                        "Subscription successful. Endpoint '{}' will receive authorisation requests notifications (cert base path: {:?})",
                        notif_endpoint, cert_base_path
                    );
                info!("{}", msg);
                Ok(json!(msg))
            }
        }
    } else {
        Err(format!(
            "Incorrect params for 'subscribe' method: {:?}",
            params
        ))
    }
}
