use hdk::prelude::*;

use hc_zome_traits::*;

use private_event_sourcing_types::{Message, ReceiveMessageInput};
use safehold_service_trait::MessageOutput;
use safehold_types::*;
use send_async_message_zome_trait::SendAsyncMessage;
use serde::de::DeserializeOwned;
use service_providers_types::*;

pub struct SafeholdAsyncMessages;

#[implement_zome_trait_as_externs]
impl SendAsyncMessage for SafeholdAsyncMessages {
    fn send_async_message(
        input: send_async_message_zome_trait::SendAsyncMessageInput,
    ) -> ExternResult<()> {
        let response = call(
            CallTargetCell::Local,
            ZomeName::from("encrypted_messages"),
            FunctionName::from("encrypt_message"),
            None,
            EncryptMessageInput {
                recipients: input.recipients.into_iter().collect(),
                message: input.message,
            },
        )?;
        let ZomeCallResponse::Ok(result) = response else {
            return Err(wasm_error!("Failed to encrypt messages: {:?}.", response));
        };

        let payload: Vec<MessageWithProvenance> = result
            .decode()
            .map_err(|err| wasm_error!("Invalid encrypt_messages result type: {:?}", err))?;
        let () = make_service_request(
            safehold_service_trait::SAFEHOLD_SERVICE_HASH.to_vec(),
            FunctionName::from("store_messages"),
            payload,
        )?;

        Ok(())
    }
}

#[hdk_extern]
pub fn receive_messages() -> ExternResult<()> {
    let encrypted_messages: Vec<MessageOutput> = make_service_request(
        safehold_service_trait::SAFEHOLD_SERVICE_HASH.to_vec(),
        FunctionName::from("get_messages"),
        (),
    )?;
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

    for decrypted_message in decrypted_messages {
        let bytes = SerializedBytes::from(UnsafeBytes::from(decrypted_message.contents));
        let message = Message::try_from(bytes)
            .map_err(|err| wasm_error!("Failed to deserialize Message bytes: {:?}.", err))?;
        let response = call(
            CallTargetCell::Local,
            ZomeName::from("messenger"),
            FunctionName::from("receive_message"),
            None,
            ReceiveMessageInput {
                provenance: decrypted_message.provenance,
                message,
            },
        )?;
        let ZomeCallResponse::Ok(_result) = response else {
            return Err(wasm_error!("Failed to receive messages: {:?}.", response));
        };
    }

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
        CallTargetCell::OtherRole("service_providers".into()),
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
        .map_err(|err| wasm_error!("Invalid get_providers_for_service response: {:?}", err))?;

    if providers.is_empty() {
        return Err(wasm_error!("There are no providers for this service."));
    }

    for service_provider in providers {
        let response = call(
            CallTargetCell::OtherRole("service_providers".into()),
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
        )?;
        let ZomeCallResponse::Ok(result) = response else {
            continue;
        };
        let Ok(deserialized_result): Result<R, SerializedBytesError> = result.decode() else {
            continue;
        };

        return Ok(deserialized_result);
    }

    Err(wasm_error!(
        "No providers were able to service the request."
    ))
}
