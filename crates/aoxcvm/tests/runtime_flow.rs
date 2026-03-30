// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcvm::context::{BlockContext, TxContext};
use aoxcvm::host::state::{HostStateView, InMemoryHostState};
use aoxcvm::lanes::VirtualMachine;
use aoxcvm::lanes::cardano::CardanoExecutor;
use aoxcvm::lanes::evm::EvmExecutor;
use aoxcvm::lanes::sui_move::SuiMoveExecutor;
use aoxcvm::lanes::wasm::WasmExecutor;
use aoxcvm::routing::dispatcher::Dispatcher;
use aoxcvm::vm_kind::VmKind;

fn block_context() -> BlockContext {
    BlockContext::new(1952, 1, 1_742_000_000, [7u8; 32], 1)
}

fn tx(
    tx_hash: [u8; 32],
    sender: &[u8],
    vm_kind: VmKind,
    payload: Vec<u8>,
    gas_limit: u64,
) -> TxContext {
    TxContext {
        tx_hash,
        sender: sender.to_vec(),
        vm_kind,
        nonce: Some(0),
        gas_limit,
        max_fee_per_gas: 1,
        payload,
        signature: vec![0xAA],
    }
}

fn dispatcher<'a>() -> Dispatcher<'a> {
    static EVM: EvmExecutor = EvmExecutor;
    static SUI: SuiMoveExecutor = SuiMoveExecutor;
    static WASM: WasmExecutor = WasmExecutor;
    static CARDANO: CardanoExecutor = CardanoExecutor;

    Dispatcher {
        evm: &EVM,
        sui_move: &SUI,
        wasm: &WASM,
        cardano: &CARDANO,
    }
}

fn canonical_lane_txs() -> Vec<TxContext> {
    vec![
        tx(
            [21u8; 32],
            b"alice",
            VmKind::Evm,
            [vec![0x00], b"deterministic-bytecode".to_vec()].concat(),
            100_000,
        ),
        tx(
            [22u8; 32],
            b"bob",
            VmKind::SuiMove,
            [vec![0x00], b"deterministic-package".to_vec()].concat(),
            100_000,
        ),
        tx(
            [23u8; 32],
            b"carol",
            VmKind::Wasm,
            [vec![0x00], b"\0asm\x01\0\0\0deterministic".to_vec()].concat(),
            100_000,
        ),
        tx(
            [24u8; 32],
            b"dave",
            VmKind::Cardano,
            [vec![0x00], b"500lovelace".to_vec()].concat(),
            100_000,
        ),
    ]
}

fn execute_all(
    router: &Dispatcher<'_>,
    block: &BlockContext,
    txs: &[TxContext],
    gas_limit: u64,
) -> (
    InMemoryHostState,
    Vec<aoxcvm::host::receipt::ExecutionReceipt>,
) {
    let mut state = InMemoryHostState::new(gas_limit);
    let mut receipts = Vec::with_capacity(txs.len());

    for tx in txs {
        receipts.push(router.execute(&mut state, block, tx).unwrap());
    }

    (state, receipts)
}

#[test]
fn evm_deploy_and_call_flow_works() {
    let block = block_context();
    let mut state = InMemoryHostState::new(1_000_000);
    let router = dispatcher();

    let deploy_tx = tx(
        [1u8; 32],
        b"alice",
        VmKind::Evm,
        [vec![0x00], b"contract-bytecode-v1".to_vec()].concat(),
        100_000,
    );

    let deploy_receipt = router.execute(&mut state, &block, &deploy_tx).unwrap();
    assert!(deploy_receipt.success);
    assert_eq!(deploy_receipt.vm_kind, VmKind::Evm);
    assert_eq!(deploy_receipt.output.len(), 20);
    assert_eq!(deploy_receipt.events.len(), 1);

    let deployed_address = deploy_receipt.output.clone();

    let call_tx = tx(
        [2u8; 32],
        b"alice",
        VmKind::Evm,
        [vec![0x01], deployed_address.clone(), b"ping".to_vec()].concat(),
        100_000,
    );

    let call_receipt = router.execute(&mut state, &block, &call_tx).unwrap();
    assert!(call_receipt.success);
    assert_eq!(call_receipt.vm_kind, VmKind::Evm);
    assert_eq!(call_receipt.output, b"ping".to_vec());
    assert_eq!(call_receipt.events.len(), 1);

    let stored = EvmExecutor
        .query(&state, &block, &deployed_address)
        .expect("deployed contract must be queryable");
    assert!(!stored.is_empty());
}

#[test]
fn sui_publish_and_object_create_flow_works() {
    let block = block_context();
    let mut state = InMemoryHostState::new(1_000_000);
    let router = dispatcher();

    let publish_tx = tx(
        [3u8; 32],
        b"bob",
        VmKind::SuiMove,
        [vec![0x00], b"move-package-v1".to_vec()].concat(),
        100_000,
    );

    let publish_receipt = router.execute(&mut state, &block, &publish_tx).unwrap();
    assert!(publish_receipt.success);
    assert_eq!(publish_receipt.vm_kind, VmKind::SuiMove);
    assert_eq!(publish_receipt.output.len(), 32);
    assert_eq!(publish_receipt.events.len(), 1);

    let object_tx = tx(
        [4u8; 32],
        b"bob",
        VmKind::SuiMove,
        [
            vec![0x01],
            vec![4u8],
            b"Coin".to_vec(),
            b"{balance:100}".to_vec(),
        ]
        .concat(),
        100_000,
    );

    let object_receipt = router.execute(&mut state, &block, &object_tx).unwrap();
    assert!(object_receipt.success);
    assert_eq!(object_receipt.vm_kind, VmKind::SuiMove);
    assert_eq!(object_receipt.output.len(), 32);
    assert_eq!(object_receipt.events.len(), 1);

    let stored_object = SuiMoveExecutor
        .query(&state, &block, &object_receipt.output)
        .expect("created object must be queryable");
    assert!(!stored_object.is_empty());
}

#[test]
fn wasm_upload_instantiate_and_execute_flow_works() {
    let block = block_context();
    let mut state = InMemoryHostState::new(1_000_000);
    let router = dispatcher();

    let upload_tx = tx(
        [5u8; 32],
        b"carol",
        VmKind::Wasm,
        [vec![0x00], b"\0asm\x01\0\0\0demo".to_vec()].concat(),
        100_000,
    );

    let upload_receipt = router.execute(&mut state, &block, &upload_tx).unwrap();
    assert!(upload_receipt.success);
    assert_eq!(upload_receipt.vm_kind, VmKind::Wasm);
    assert_eq!(upload_receipt.output.len(), 32);
    assert_eq!(upload_receipt.events.len(), 1);

    let code_id = upload_receipt.output.clone();

    let instantiate_tx = tx(
        [6u8; 32],
        b"carol",
        VmKind::Wasm,
        [vec![0x01], code_id.clone(), b"state:v1".to_vec()].concat(),
        100_000,
    );

    let instantiate_receipt = router.execute(&mut state, &block, &instantiate_tx).unwrap();
    assert!(instantiate_receipt.success);
    assert_eq!(instantiate_receipt.vm_kind, VmKind::Wasm);
    assert_eq!(instantiate_receipt.output.len(), 32);
    assert_eq!(instantiate_receipt.events.len(), 1);

    let instance_id = instantiate_receipt.output.clone();

    let execute_tx = tx(
        [7u8; 32],
        b"carol",
        VmKind::Wasm,
        [vec![0x02], instance_id.clone(), b"state:v2".to_vec()].concat(),
        100_000,
    );

    let execute_receipt = router.execute(&mut state, &block, &execute_tx).unwrap();
    assert!(execute_receipt.success);
    assert_eq!(execute_receipt.vm_kind, VmKind::Wasm);
    assert_eq!(execute_receipt.output, b"state:v2".to_vec());
    assert_eq!(execute_receipt.events.len(), 1);

    let stored_instance = WasmExecutor
        .query(&state, &block, &instance_id)
        .expect("instance must be queryable");
    assert!(!stored_instance.is_empty());
}

#[test]
fn wasm_upload_rejects_invalid_header() {
    let block = block_context();
    let mut state = InMemoryHostState::new(1_000_000);
    let router = dispatcher();

    let invalid_upload = tx(
        [35u8; 32],
        b"carol",
        VmKind::Wasm,
        [vec![0x00], b"not-wasm".to_vec()].concat(),
        100_000,
    );

    let err = router
        .execute(&mut state, &block, &invalid_upload)
        .expect_err("invalid WASM header must be rejected");
    assert!(err.to_string().contains("WASM module"));
}

#[test]
fn wasm_instantiate_rejects_oversized_state() {
    let block = block_context();
    let mut state = InMemoryHostState::new(5_000_000);
    let router = dispatcher();

    let upload_tx = tx(
        [36u8; 32],
        b"carol",
        VmKind::Wasm,
        [vec![0x00], b"\0asm\x01\0\0\0state-size".to_vec()].concat(),
        200_000,
    );
    let upload_receipt = router.execute(&mut state, &block, &upload_tx).unwrap();

    let oversized_state = vec![0xAB; 262_145];
    let instantiate_tx = tx(
        [37u8; 32],
        b"carol",
        VmKind::Wasm,
        [vec![0x01], upload_receipt.output, oversized_state].concat(),
        500_000,
    );

    let err = router
        .execute(&mut state, &block, &instantiate_tx)
        .expect_err("oversized state must be rejected");
    assert!(err.to_string().contains("state exceeds max size"));
}

#[test]
fn cardano_utxo_create_and_spend_flow_works() {
    let block = block_context();
    let mut state = InMemoryHostState::new(1_000_000);
    let router = dispatcher();

    let create_tx = tx(
        [8u8; 32],
        b"dave",
        VmKind::Cardano,
        [vec![0x00], b"1000lovelace".to_vec()].concat(),
        100_000,
    );

    let create_receipt = router.execute(&mut state, &block, &create_tx).unwrap();
    assert!(create_receipt.success);
    assert_eq!(create_receipt.vm_kind, VmKind::Cardano);
    assert_eq!(create_receipt.output.len(), 32);
    assert_eq!(create_receipt.events.len(), 1);

    let utxo_id = create_receipt.output.clone();

    let spend_tx = tx(
        [9u8; 32],
        b"dave",
        VmKind::Cardano,
        [vec![0x01], utxo_id.clone()].concat(),
        100_000,
    );

    let spend_receipt = router.execute(&mut state, &block, &spend_tx).unwrap();
    assert!(spend_receipt.success);
    assert_eq!(spend_receipt.vm_kind, VmKind::Cardano);
    assert_eq!(spend_receipt.output, b"1000lovelace".to_vec());
    assert_eq!(spend_receipt.events.len(), 1);

    let missing = CardanoExecutor.query(&state, &block, &utxo_id);
    assert!(missing.is_err());
}

#[test]
fn dispatcher_and_gas_accounting_work_across_all_lanes() {
    let block = block_context();
    let mut state = InMemoryHostState::new(1_000_000);
    let router = dispatcher();

    let txs = vec![
        tx(
            [10u8; 32],
            b"alice",
            VmKind::Evm,
            [vec![0x00], b"bytecode".to_vec()].concat(),
            100_000,
        ),
        tx(
            [11u8; 32],
            b"bob",
            VmKind::SuiMove,
            [vec![0x00], b"pkg".to_vec()].concat(),
            100_000,
        ),
        tx(
            [12u8; 32],
            b"carol",
            VmKind::Wasm,
            [vec![0x00], b"\0asm\x01\0\0\0gas".to_vec()].concat(),
            100_000,
        ),
        tx(
            [13u8; 32],
            b"dave",
            VmKind::Cardano,
            [vec![0x00], b"value".to_vec()].concat(),
            100_000,
        ),
    ];

    let initial_gas = state.gas_remaining();

    for tx in &txs {
        let receipt = router.execute(&mut state, &block, tx).unwrap();
        assert!(receipt.success);
    }

    assert!(state.gas_remaining() < initial_gas);
    assert!(state.len() >= 4);
}

#[test]
fn gas_exhaustion_is_enforced() {
    let block = block_context();
    let mut state = InMemoryHostState::new(10_000);
    let router = dispatcher();

    let deploy_tx = tx(
        [14u8; 32],
        b"alice",
        VmKind::Evm,
        [vec![0x00], b"too-expensive".to_vec()].concat(),
        100_000,
    );

    let err = router.execute(&mut state, &block, &deploy_tx).unwrap_err();
    let rendered = err.to_string();
    assert!(rendered.contains("gas exhausted"));
}

#[test]
fn multilane_gas_accounting_is_deterministic_across_identical_runs() {
    let block = block_context();
    let router = dispatcher();
    let txs = canonical_lane_txs();

    let (state_a, receipts_a) = execute_all(&router, &block, &txs, 1_000_000);
    let (state_b, receipts_b) = execute_all(&router, &block, &txs, 1_000_000);

    let gas_profile_a: Vec<(VmKind, u64)> = receipts_a
        .iter()
        .map(|receipt| (receipt.vm_kind, receipt.gas_used))
        .collect();
    let gas_profile_b: Vec<(VmKind, u64)> = receipts_b
        .iter()
        .map(|receipt| (receipt.vm_kind, receipt.gas_used))
        .collect();

    assert_eq!(gas_profile_a, gas_profile_b);
    assert_eq!(state_a.gas_remaining(), state_b.gas_remaining());
    assert_eq!(state_a.raw_storage(), state_b.raw_storage());
}

#[test]
fn each_lane_enforces_the_same_host_resource_limit_boundary() {
    let block = block_context();
    let router = dispatcher();

    for tx in canonical_lane_txs() {
        let mut calibration_state = InMemoryHostState::new(1_000_000);
        let receipt = router.execute(&mut calibration_state, &block, &tx).unwrap();
        let required_gas = receipt.gas_used;

        let mut insufficient_state = InMemoryHostState::new(required_gas.saturating_sub(1));
        let err = router
            .execute(&mut insufficient_state, &block, &tx)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "gas exhausted",
            "lane {:?} crossed an inconsistent exhaustion boundary",
            tx.vm_kind
        );

        let mut exact_state = InMemoryHostState::new(required_gas);
        let exact_receipt = router.execute(&mut exact_state, &block, &tx).unwrap();
        assert_eq!(exact_receipt.gas_used, required_gas);
        assert_eq!(exact_state.gas_remaining(), 0);
    }
}

#[test]
fn same_logical_identifier_can_exist_in_multiple_lanes_without_state_collision() {
    let block = block_context();
    let mut state = InMemoryHostState::new(1_000_000);
    let router = dispatcher();
    let shared_id = [31u8; 32];

    let evm_deploy = tx(
        shared_id,
        b"alice",
        VmKind::Evm,
        [vec![0x00], b"shared-id-bytecode".to_vec()].concat(),
        100_000,
    );
    let sui_publish = tx(
        shared_id,
        b"bob",
        VmKind::SuiMove,
        [vec![0x00], b"shared-id-package".to_vec()].concat(),
        100_000,
    );
    let wasm_upload = tx(
        shared_id,
        b"carol",
        VmKind::Wasm,
        [vec![0x00], b"\0asm\x01\0\0\0shared-id".to_vec()].concat(),
        100_000,
    );
    let cardano_create = tx(
        shared_id,
        b"dave",
        VmKind::Cardano,
        [vec![0x00], b"shared-id-utxo".to_vec()].concat(),
        100_000,
    );

    let evm_receipt = router.execute(&mut state, &block, &evm_deploy).unwrap();
    let sui_receipt = router.execute(&mut state, &block, &sui_publish).unwrap();
    let wasm_receipt = router.execute(&mut state, &block, &wasm_upload).unwrap();
    let cardano_receipt = router.execute(&mut state, &block, &cardano_create).unwrap();

    assert_eq!(sui_receipt.output, shared_id.to_vec());
    assert_eq!(wasm_receipt.output, shared_id.to_vec());
    assert_eq!(cardano_receipt.output, shared_id.to_vec());
    assert_ne!(
        evm_receipt.output,
        shared_id.to_vec(),
        "EVM address derivation should stay lane-specific"
    );

    assert!(
        EvmExecutor
            .query(&state, &block, &evm_receipt.output)
            .is_ok()
    );
    assert!(SuiMoveExecutor.query(&state, &block, &shared_id).is_ok());
    assert!(WasmExecutor.query(&state, &block, &shared_id).is_ok());
    assert!(CardanoExecutor.query(&state, &block, &shared_id).is_ok());

    let storage_keys: Vec<Vec<u8>> = state.raw_storage().keys().cloned().collect();
    assert!(storage_keys.iter().any(|key| key.starts_with(b"evm/")));
    assert!(storage_keys.iter().any(|key| key.starts_with(b"sui/")));
    assert!(storage_keys.iter().any(|key| key.starts_with(b"wasm/")));
    assert!(storage_keys.iter().any(|key| key.starts_with(b"cardano/")));
}
