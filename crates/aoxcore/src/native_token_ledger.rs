impl NativeTokenLedger {
    /// Constructs a new ledger from the supplied policy.
    pub fn new(policy: NativeTokenPolicy) -> Result<Self, NativeTokenError> {
        policy.validate()?;

        Ok(Self {
            policy,
            total_supply: 0,
            balances: HashMap::new(),
            latest_nonce: HashMap::new(),
            consumed_quantum_commitments: HashSet::new(),
        })
    }

    /// Constructs a new ledger using the canonical policy for the selected network.
    pub fn new_for_network(network: NativeTokenNetwork) -> Result<Self, NativeTokenError> {
        Self::new(NativeTokenPolicy::for_network(network))
    }

    /// Returns the balance for the requested address.
    #[must_use]
    pub fn balance_of(&self, address: &Address) -> u128 {
        self.balances.get(address).copied().unwrap_or(0)
    }

    /// Returns the last accepted sender nonce, when present.
    #[must_use]
    pub fn latest_nonce_of(&self, address: &Address) -> Option<u64> {
        self.latest_nonce.get(address).copied()
    }

    /// Returns whether the supplied quantum commitment was already consumed.
    #[must_use]
    pub fn has_consumed_quantum_commitment(
        &self,
        digest: &[u8; NATIVE_TOKEN_COMMITMENT_SIZE],
    ) -> bool {
        self.consumed_quantum_commitments.contains(digest)
    }

    /// Mints native tokens into the supplied destination account.
    pub fn mint(&mut self, to: Address, amount: u128) -> Result<(), NativeTokenError> {
        self.policy.validate()?;

        if !self.policy.allows_mint() {
            return Err(NativeTokenError::MintDisabledPolicy);
        }

        self.policy.validate_mint_amount(amount)?;

        let updated_supply = self
            .total_supply
            .checked_add(amount)
            .ok_or(NativeTokenError::SupplyOverflow)?;

        if updated_supply > self.policy.quantum_policy.max_total_supply {
            return Err(NativeTokenError::SupplyOverflow);
        }

        let current_balance = self.balance_of(&to);
        let updated_balance = current_balance
            .checked_add(amount)
            .ok_or(NativeTokenError::BalanceOverflow)?;

        self.total_supply = updated_supply;
        self.balances.insert(to, updated_balance);

        Ok(())
    }

    /// Transfers native tokens between two accounts.
    pub fn transfer(
        &mut self,
        from: Address,
        to: Address,
        amount: u128,
    ) -> Result<(), NativeTokenError> {
        self.policy.validate()?;
        self.policy.validate_transfer_amount(amount)?;

        let current_from_balance = self.balance_of(&from);
        if current_from_balance < amount {
            return Err(NativeTokenError::InsufficientBalance);
        }

        let remaining_from_balance = current_from_balance - amount;
        if remaining_from_balance == 0 {
            self.balances.remove(&from);
        } else {
            self.balances.insert(from, remaining_from_balance);
        }

        let current_to_balance = self.balance_of(&to);
        let updated_to_balance = current_to_balance
            .checked_add(amount)
            .ok_or(NativeTokenError::BalanceOverflow)?;
        self.balances.insert(to, updated_to_balance);

        Ok(())
    }

    /// Executes a replay-hardened transfer with explicit quantum-proof binding.
    ///
    /// Replay protection currently includes:
    /// - strict sender nonce monotonicity,
    /// - non-empty proof tag validation,
    /// - proof-tag size enforcement,
    /// - full commitment digest tracking under the configured replay domain.
    pub fn transfer_quantum(
        &mut self,
        from: Address,
        to: Address,
        amount: u128,
        nonce: u64,
        proof_tag: &[u8],
    ) -> Result<(), NativeTokenError> {
        self.policy.validate()?;
        self.policy.validate_transfer_amount(amount)?;
        self.policy.validate_proof_tag(proof_tag)?;

        match self.latest_nonce.get(&from).copied() {
            Some(last_nonce) if nonce < last_nonce => {
                return Err(NativeTokenError::NonceRegression);
            }
            Some(last_nonce) if nonce == last_nonce => {
                return Err(NativeTokenError::ReplayDetected);
            }
            _ => {}
        }

        let commitment = self.quantum_transfer_digest(from, to, amount, nonce, proof_tag);

        if self
            .consumed_quantum_commitments
            .contains(&commitment.digest)
        {
            return Err(NativeTokenError::ReplayDetected);
        }

        self.transfer(from, to, amount)?;

        self.latest_nonce.insert(from, nonce);
        self.consumed_quantum_commitments.insert(commitment.digest);

        Ok(())
    }

    /// Computes the canonical quantum transfer digest under the active policy domain.
    #[must_use]
    pub fn quantum_transfer_digest(
        &self,
        from: Address,
        to: Address,
        amount: u128,
        nonce: u64,
        proof_tag: &[u8],
    ) -> NativeQuantumTransferDigestV1 {
        NativeQuantumTransferDigestV1 {
            version: NATIVE_TOKEN_QUANTUM_EVENT_VERSION,
            digest: compute_quantum_transfer_digest(
                &self.policy.quantum_policy.anti_replay_domain,
                from,
                to,
                amount,
                nonce,
                proof_tag,
            ),
        }
    }

    /// Builds a receipt for a successful mint operation.
    ///
    /// This method is compatible with the hardened receipt API and therefore
    /// returns `Result` instead of constructing invalid states implicitly.
    pub fn mint_receipt(
        &self,
        tx_hash: [u8; HASH_SIZE],
        to: Address,
        amount: u128,
        energy_used: u64,
    ) -> Result<Receipt, ReceiptError> {
        let mut receipt = Receipt::success(tx_hash, energy_used)?;

        let event = Event::new(
            EVENT_NATIVE_MINT,
            encode_transfer_like_event([0u8; 32], to, amount),
        )?;
        receipt.push_event(event)?;

        Ok(receipt)
    }

    /// Builds a receipt for a successful classic native transfer operation.
    pub fn transfer_receipt(
        &self,
        tx_hash: [u8; HASH_SIZE],
        from: Address,
        to: Address,
        amount: u128,
        energy_used: u64,
    ) -> Result<Receipt, ReceiptError> {
        let mut receipt = Receipt::success(tx_hash, energy_used)?;

        let event = Event::new(
            EVENT_NATIVE_TRANSFER,
            encode_transfer_like_event(from, to, amount),
        )?;
        receipt.push_event(event)?;

        Ok(receipt)
    }

    /// Builds a receipt for a failed native token operation.
    pub fn error_receipt(
        &self,
        tx_hash: [u8; HASH_SIZE],
        energy_used: u64,
        error: NativeTokenError,
    ) -> Result<Receipt, ReceiptError> {
        Receipt::failure(tx_hash, energy_used, error.receipt_error_code())
    }

    /// Builds a receipt for a successful replay-hardened quantum transfer.
    #[allow(clippy::too_many_arguments)]
    pub fn transfer_quantum_receipt(
        &self,
        tx_hash: [u8; HASH_SIZE],
        from: Address,
        to: Address,
        amount: u128,
        nonce: u64,
        proof_tag: &[u8],
        energy_used: u64,
    ) -> Result<Receipt, ReceiptError> {
        let mut receipt = Receipt::success(tx_hash, energy_used)?;

        let event = Event::new(
            EVENT_NATIVE_TRANSFER_QUANTUM_V1,
            encode_quantum_transfer_event_v1(
                &self.policy.quantum_policy.anti_replay_domain,
                from,
                to,
                amount,
                nonce,
                proof_tag,
            ),
        )?;
        receipt.push_event(event)?;

        Ok(receipt)
    }
}

/// Encodes a classic transfer-like event payload.
///
/// Layout:
/// - from: 32 bytes
/// - to: 32 bytes
/// - amount: 16 bytes little-endian
#[must_use]
pub fn encode_transfer_like_event(from: Address, to: Address, amount: u128) -> Vec<u8> {
    let mut payload = Vec::with_capacity(80);
    payload.extend_from_slice(&from);
    payload.extend_from_slice(&to);
    payload.extend_from_slice(&amount.to_le_bytes());
    payload
}

/// Encodes a versioned quantum transfer event payload.
///
/// Layout:
/// - version: 1 byte
/// - from: 32 bytes
/// - to: 32 bytes
/// - amount: 16 bytes little-endian
/// - nonce: 8 bytes little-endian
/// - digest: 32 bytes
#[must_use]
pub fn encode_quantum_transfer_event_v1(
    domain: &str,
    from: Address,
    to: Address,
    amount: u128,
    nonce: u64,
    proof_tag: &[u8],
) -> Vec<u8> {
    let digest = compute_quantum_transfer_digest(domain, from, to, amount, nonce, proof_tag);

    let mut payload = Vec::with_capacity(89);
    payload.push(NATIVE_TOKEN_QUANTUM_EVENT_VERSION);
    payload.extend_from_slice(&from);
    payload.extend_from_slice(&to);
    payload.extend_from_slice(&amount.to_le_bytes());
    payload.extend_from_slice(&nonce.to_le_bytes());
    payload.extend_from_slice(&digest);
    payload
}

/// Backward-compatible alias for callers still using the previous helper name.
///
/// The implementation emits the versioned V1 quantum event layout.
#[must_use]
pub fn encode_quantum_transfer_event(
    domain: &str,
    from: Address,
    to: Address,
    amount: u128,
    nonce: u64,
    proof_tag: &[u8],
) -> Vec<u8> {
    encode_quantum_transfer_event_v1(domain, from, to, amount, nonce, proof_tag)
}

/// Computes the canonical replay-binding digest for a quantum transfer.
#[must_use]
pub fn compute_quantum_transfer_digest(
    domain: &str,
    from: Address,
    to: Address,
    amount: u128,
    nonce: u64,
    proof_tag: &[u8],
) -> [u8; NATIVE_TOKEN_COMMITMENT_SIZE] {
    let mut hasher = Sha3_256::new();
    hasher.update(domain.as_bytes());
    hasher.update([0x00]);
    hasher.update(from);
    hasher.update([0x00]);
    hasher.update(to);
    hasher.update([0x00]);
    hasher.update(amount.to_le_bytes());
    hasher.update([0x00]);
    hasher.update(nonce.to_le_bytes());
    hasher.update([0x00]);
    hasher.update(proof_tag);

    let digest = hasher.finalize();

    let mut out = [0u8; NATIVE_TOKEN_COMMITMENT_SIZE];
    out.copy_from_slice(&digest[..NATIVE_TOKEN_COMMITMENT_SIZE]);
    out
}

