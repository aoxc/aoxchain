use aoxcvm::{
    receipts::outcome::{ExecutionReceipt, ReceiptStatus},
    state::JournaledState,
};

fn main() {
    let mut state = JournaledState::default();
    state.put(b"alpha".to_vec(), b"1".to_vec());
    state.put(b"beta".to_vec(), b"2".to_vec());

    let receipt = ExecutionReceipt::from_state(
        ReceiptStatus::Success,
        4242,
        vec!["determinism-probe".to_string()],
        &state,
    );

    for byte in receipt.state_root {
        print!("{byte:02x}");
    }
    println!();
}
