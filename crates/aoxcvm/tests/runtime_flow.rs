use aovm::context::{BlockContext, TxContext};
use aovm::host::state::{HostStateView, InMemoryHostState};
use aovm::lanes::cardano::CardanoExecutor;
use aovm::lanes::evm::EvmExecutor;
use aovm::lanes::sui_move::SuiMoveExecutor;
use aovm::lanes::wasm::WasmExecutor;
use aovm::lanes::VirtualMachine;
use aovm::routing::dispatcher::Dispatcher;
use aovm::vm_kind::VmKind;

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
        [
            vec![0x01],
            deployed_address.clone(),
            b"ping".to_vec(),
        ]
        .concat(),
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
        [vec![0x00], b"\0asm-demo".to_vec()].concat(),
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
        [
            vec![0x01],
            code_id.clone(),
            b"state:v1".to_vec(),
        ]
        .concat(),
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
        [
            vec![0x02],
            instance_id.clone(),
            b"state:v2".to_vec(),
        ]
        .concat(),
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
            [vec![0x00], b"wasm".to_vec()].concat(),
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
