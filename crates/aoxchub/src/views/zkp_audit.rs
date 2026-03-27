use crate::components::glass::GlassSurface;
use crate::services::rpc_client::RpcClient;
use dioxus::prelude::*;

#[component]
pub fn ZkpAudit() -> Element {
    let client = use_context::<RpcClient>();

    let mut actor_id = use_signal(|| "validator-01".to_string());
    let mut payload_hex = use_signal(String::new);
    let mut proof_hex = use_signal(String::new);
    let mut submit_result = use_signal(|| None::<String>);

    rsx! {
        div { class: "space-y-4",
            h2 { class: "text-2xl font-bold text-white", "ZKP Audit / Tx Submission" }
            p { class: "text-slate-300", "Node HTTP yüzeyi `POST /api/v1/tx/submit` sağlıyorsa gerçek gönderim yapılır." }

            GlassSurface { class: Some("p-5 space-y-3".to_string()),
                input {
                    class: "w-full rounded-lg border border-white/15 bg-black/30 p-2 text-sm text-white",
                    value: actor_id(),
                    placeholder: "actor_id",
                    oninput: move |e| actor_id.set(e.value()),
                }
                textarea {
                    class: "w-full rounded-lg border border-white/15 bg-black/30 p-2 text-sm text-white",
                    rows: 4,
                    value: payload_hex(),
                    placeholder: "tx_payload (hex, 0x...)",
                    oninput: move |e| payload_hex.set(e.value()),
                }
                textarea {
                    class: "w-full rounded-lg border border-white/15 bg-black/30 p-2 text-sm text-white",
                    rows: 4,
                    value: proof_hex(),
                    placeholder: "zkp_proof (hex, 0x...)",
                    oninput: move |e| proof_hex.set(e.value()),
                }
                button {
                    class: "rounded-lg border border-blue-400/40 bg-blue-500/15 px-3 py-1 text-sm text-blue-100",
                    onclick: {
                        let client = client.clone();
                        let actor = actor_id();
                        let payload = payload_hex();
                        let proof = proof_hex();
                        move |_| {
                            spawn(async move {
                                let result = submit_zkp_tx(&client, &actor, &payload, &proof).await;
                                submit_result.set(Some(result));
                            });
                        }
                    },
                    "Gönder"
                }
                if let Some(result) = submit_result() {
                    pre { class: "overflow-x-auto rounded-lg bg-black/30 p-3 text-xs text-slate-200", "{result}" }
                }
            }
        }
    }
}

async fn submit_zkp_tx(
    client: &RpcClient,
    actor_id: &str,
    payload_hex: &str,
    proof_hex: &str,
) -> String {
    let payload = match decode_hex(payload_hex) {
        Ok(bytes) => bytes,
        Err(err) => return err,
    };

    let proof = match decode_hex(proof_hex) {
        Ok(bytes) => bytes,
        Err(err) => return err,
    };

    match client.submit_zkp_tx(actor_id, payload, proof).await {
        Ok(json) => serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string()),
        Err(err) => err,
    }
}

fn decode_hex(input: &str) -> Result<Vec<u8>, String> {
    let clean = input.trim_start_matches("0x");
    if clean.len() % 2 != 0 {
        return Err("hex uzunluğu çift olmalı".to_string());
    }

    (0..clean.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&clean[i..i + 2], 16).map_err(|e| format!("hex parse hatası: {e}"))
        })
        .collect()
}
