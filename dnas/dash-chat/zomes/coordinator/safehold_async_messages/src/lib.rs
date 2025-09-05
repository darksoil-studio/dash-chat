use hdk::prelude::*;

use hc_zome_traits::*;

use private_event_sourcing_types::{Message, ReceiveMessageWithProvenanceInput};
use push_notifications_types::{PushNotification, SendPushNotificationToAgentInput};
use safehold_service_trait::MessageOutput;
use safehold_types::*;
use send_async_message_zome_trait::SendAsyncMessage;
use serde::de::DeserializeOwned;
use service_providers_types::*;

pub struct SafeholdAsyncMessages;

#[implemented_zome_traits]
pub enum ZomeTraits {
    SendAsyncMessage(SafeholdAsyncMessages),
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes)]
pub struct MessageWithZome {
    zome_name: ZomeName,
    message: Vec<u8>,
}

#[implement_zome_trait_as_externs]
impl SendAsyncMessage for SafeholdAsyncMessages {
    fn send_async_message(
        input: send_async_message_zome_trait::SendAsyncMessageInput,
    ) -> ExternResult<()> {
        let m = MessageWithZome {
            zome_name: input.zome_name.clone(),
            message: input.message,
        };
        let sb = SerializedBytes::try_from(m).map_err(|err| wasm_error!(err))?;
        let bytes = sb.bytes().to_vec();

        let response = call(
            CallTargetCell::Local,
            ZomeName::from("encrypted_messages"),
            FunctionName::from("encrypt_message"),
            None,
            EncryptMessageInput {
                recipients: input.recipients.iter().cloned().collect(),
                message: bytes,
            },
        )?;
        let ZomeCallResponse::Ok(result) = response else {
            return Err(wasm_error!("Failed to encrypt messages: {:?}.", response));
        };

        let payload: Vec<MessageWithProvenance> = result
            .decode()
            .map_err(|err| wasm_error!("Invalid encrypt_messages result type: {:?}", err))?;

        info!(
            "Storing {} messages in the safehold service.",
            payload.len()
        );
        let () = make_service_request(
            safehold_service_trait::SAFEHOLD_SERVICE_HASH.to_vec(),
            FunctionName::from("store_messages"),
            payload,
        )?;
        info!("Successfully stored messages.");

        let send_push_notifications_input: Vec<SendPushNotificationToAgentInput> = input
            .recipients
            .into_iter()
            .map(|r| SendPushNotificationToAgentInput {
                agent: r,
                notification: PushNotification {
                    title: input.zome_name.clone().to_string(),
                    body: input.message_id.clone(),
                },
            })
            .collect();

        info!("Sending push notification.");

        let () = make_service_request(
            push_notifications_service_trait::PUSH_NOTIFICATIONS_SERVICE_HASH.to_vec(),
            FunctionName::from("send_push_notifications"),
            send_push_notifications_input,
        )?;

        info!("Successfully sent push notification.");

        Ok(())
    }
}

#[hdk_extern]
pub fn receive_messages() -> ExternResult<()> {
    debug!("[receive_messages] start.");
    let encrypted_messages: Vec<MessageOutput> = make_service_request(
        safehold_service_trait::SAFEHOLD_SERVICE_HASH.to_vec(),
        FunctionName::from("get_messages"),
        (),
    )?;

    if encrypted_messages.is_empty() {
        debug!("[receive_messages] no messages for me.",);
        return Ok(());
    }

    debug!(
        "[receive_messages] received {} messages, decrypting.",
        encrypted_messages.len()
    );

    let response = call(
        CallTargetCell::Local,
        ZomeName::from("encrypted_messages"),
        FunctionName::from("decrypt_messages"),
        None,
        encrypted_messages,
    )?;
    let ZomeCallResponse::Ok(result) = response else {
        return Err(wasm_error!("Failed to receive messages: {:?}.", response));
    };
    let decrypted_messages: Vec<DecryptedMessageOutput> = result
        .decode()
        .map_err(|err| wasm_error!("Failed to parse decrypt_message result: {:?}.", err))?;

    debug!("[receive_messages] Successfully decrypted messages.",);

    for decrypted_message in decrypted_messages {
        let bytes = SerializedBytes::from(UnsafeBytes::from(decrypted_message.contents));
        let message_with_zome = MessageWithZome::try_from(bytes)
            .map_err(|err| wasm_error!("Failed to deserialize Message bytes: {:?}.", err))?;
        let bytes = SerializedBytes::from(UnsafeBytes::from(message_with_zome.message));
        let message = Message::try_from(bytes)
            .map_err(|err| wasm_error!("Failed to deserialize Message bytes: {:?}.", err))?;
        let response = call_remote(
            agent_info()?.agent_initial_pubkey, // Move to call when https://github.com/holochain/holochain/issues/5123 is solved
            message_with_zome.zome_name,
            FunctionName::from("receive_message_with_provenance"),
            None,
            ReceiveMessageWithProvenanceInput {
                provenance: decrypted_message.provenance,
                message,
            },
        )?;
        let ZomeCallResponse::Ok(_result) = response else {
            return Err(wasm_error!("Failed to receive messages: {:?}.", response));
        };
    }

    debug!("[receive_messages] Successfully stored decrypted messages.",);

    Ok(())
}

fn make_service_request<P, R>(
    service_id: ServiceId,
    fn_name: FunctionName,
    payload: P,
) -> ExternResult<R>
where
    R: Serialize + DeserializeOwned + std::fmt::Debug,
    P: Serialize + DeserializeOwned + std::fmt::Debug,
{
    let response = call(
        CallTargetCell::OtherRole("services".into()),
        ZomeName::from("service_providers"),
        FunctionName::from("get_providers_for_service"),
        None,
        service_id.clone(),
    )?;
    let ZomeCallResponse::Ok(result) = response else {
        return Err(wasm_error!(
            "Failed to get service providers: {:?}.",
            response
        ));
    };
    let providers: Vec<AgentPubKey> = result
        .decode()
        .map_err(|err| wasm_error!("Invalid get_providers_for_service response: {:?}.", err))?;

    if providers.is_empty() {
        return Err(wasm_error!("There are no providers for this service."));
    }

    for service_provider in providers {
        let response = call(
            CallTargetCell::OtherRole("services".into()),
            ZomeName::from("service_providers"),
            FunctionName::from("make_service_request"),
            None,
            MakeServiceRequestInput {
                service_id: service_id.clone(),
                service_provider,
                fn_name: fn_name.clone(),
                payload: ExternIO::encode(&payload)
                    .map_err(|err| wasm_error!("Failed to serialize payload: {:?}.", err))?,
            },
        );
        let Ok(ZomeCallResponse::Ok(result)) = response else {
            warn!(
                "Error calling make_service_request with function: {}.",
                fn_name
            );
            continue;
        };

        match result.decode::<ExternIO>() {
            Ok(deserialized_result) => match deserialized_result.decode::<R>() {
                Ok(deserialized_result) => {
                    return Ok(deserialized_result);
                }
                Err(err) => {
                    error!(
                        "Error deserializing result for function {}: {:?}.",
                        fn_name, err
                    );
                }
            },
            Err(err) => {
                error!(
                    "Error deserializing result for function {}: {:?}.",
                    fn_name, err
                );
            }
        }
    }

    Err(wasm_error!(
        "No providers were able to service the request for function {}.",
        fn_name
    ))
}
