impl Certificate {
    /// Creates a new unsigned certificate.
    ///
    /// `issuer` and `signature` are initialized empty so that issuance
    /// workflows may populate them later after canonical validation.
    #[must_use]
    pub fn new_unsigned(
        chain: String,
        actor_id: String,
        role: String,
        zone: String,
        pubkey: String,
        issued_at: u64,
        expires_at: u64,
    ) -> Self {
        Self {
            version: CERTIFICATE_VERSION,
            chain,
            actor_id,
            role,
            zone,
            pubkey,
            issued_at,
            expires_at,
            issuer: String::new(),
            signature: String::new(),
        }
    }

    /// Returns the canonical signing payload for this certificate.
    ///
    /// The returned payload excludes the detached signature field.
    #[must_use]
    pub fn signing_payload(&self) -> CertificateSigningPayload {
        CertificateSigningPayload {
            version: self.version,
            chain: self.chain.clone(),
            actor_id: self.actor_id.clone(),
            role: self.role.clone(),
            zone: self.zone.clone(),
            pubkey: self.pubkey.clone(),
            issued_at: self.issued_at,
            expires_at: self.expires_at,
            issuer: self.issuer.clone(),
        }
    }

    /// Serializes the canonical signing payload into JSON bytes.
    ///
    /// The payload is validated before serialization so that callers cannot
    /// accidentally sign semantically invalid certificate content.
    pub fn signing_payload_bytes(&self) -> Result<Vec<u8>, CertificateError> {
        let payload = self.signing_payload();
        payload.validate()?;

        serde_json::to_vec(&payload)
            .map_err(|error| CertificateError::SerializationFailed(error.to_string()))
    }

    /// Returns a copy of the certificate with the signature field cleared.
    #[must_use]
    pub fn unsigned_view(&self) -> Self {
        let mut cloned = self.clone();
        cloned.signature.clear();
        cloned
    }

    /// Returns whether the certificate currently carries a non-blank signature.
    #[must_use]
    pub fn is_signed(&self) -> bool {
        !self.signature.trim().is_empty()
    }

    /// Returns whether the certificate currently carries a non-blank issuer.
    #[must_use]
    pub fn has_issuer(&self) -> bool {
        !self.issuer.trim().is_empty()
    }

    /// Returns the decoded public-key bytes after validation.
    pub fn public_key_bytes(&self) -> Result<Vec<u8>, CertificateError> {
        validate_pubkey_hex(&self.pubkey)?;
        hex::decode(self.pubkey.trim()).map_err(|_| CertificateError::InvalidPublicKeyHex)
    }

    /// Returns the decoded detached-signature bytes after validation.
    pub fn signature_bytes(&self) -> Result<Vec<u8>, CertificateError> {
        validate_signature_hex(&self.signature)?;
        hex::decode(self.signature.trim()).map_err(|_| CertificateError::InvalidSignatureHex)
    }

    /// Returns a deterministic operator-facing fingerprint of the certificate.
    ///
    /// The fingerprint is derived from the full serialized certificate object,
    /// including issuer and signature fields. A domain separator is applied to
    /// prevent accidental reuse across unrelated digest contexts.
    pub fn fingerprint(&self) -> Result<String, CertificateError> {
        let body = serde_json::to_vec(self)
            .map_err(|error| CertificateError::SerializationFailed(error.to_string()))?;

        let mut hasher = Sha3_256::new();
        hasher.update(CERTIFICATE_FINGERPRINT_DOMAIN);
        hasher.update([0x00]);
        hasher.update(body);

        let digest = hasher.finalize();
        Ok(hex::encode_upper(&digest[..8]))
    }

    /// Returns the certificate lifecycle classification at the supplied UNIX timestamp.
    #[must_use]
    pub fn validity_state_at(&self, unix_time: u64) -> CertificateValidityState {
        if self.is_not_yet_valid_at(unix_time) {
            CertificateValidityState::NotYetValid
        } else if self.is_expired_at(unix_time) {
            CertificateValidityState::Expired
        } else {
            CertificateValidityState::Valid
        }
    }

    /// Validates the certificate fields required before signing.
    ///
    /// This validation intentionally does not require `issuer` or `signature`
    /// because unsigned certificates are valid intermediate objects during
    /// issuance workflows.
    pub fn validate_unsigned(&self) -> Result<(), CertificateError> {
        if self.version != CERTIFICATE_VERSION {
            return Err(CertificateError::InvalidVersion);
        }

        validate_chain(&self.chain)?;
        validate_actor_id(&self.actor_id)?;
        validate_role(&self.role)?;
        validate_zone(&self.zone)?;
        validate_pubkey_hex(&self.pubkey)?;
        validate_validity_window(self.issued_at, self.expires_at)?;

        Ok(())
    }

    /// Validates the fully issued certificate.
    ///
    /// This includes:
    /// - unsigned payload validation,
    /// - issuer validation,
    /// - signature presence and encoding validation.
    pub fn validate_signed(&self) -> Result<(), CertificateError> {
        self.validate_unsigned()?;
        validate_issuer(&self.issuer)?;
        validate_signature_hex(&self.signature)?;

        Ok(())
    }

    /// Returns whether the certificate is valid at the supplied UNIX timestamp.
    #[must_use]
    pub fn is_valid_at(&self, unix_time: u64) -> bool {
        unix_time >= self.issued_at && unix_time < self.expires_at
    }

    /// Returns whether the certificate is expired at the supplied UNIX timestamp.
    #[must_use]
    pub fn is_expired_at(&self, unix_time: u64) -> bool {
        unix_time >= self.expires_at
    }

    /// Returns whether the certificate is not yet valid at the supplied UNIX timestamp.
    #[must_use]
    pub fn is_not_yet_valid_at(&self, unix_time: u64) -> bool {
        unix_time < self.issued_at
    }

    /// Returns whether the certificate is currently valid according to system time.
    pub fn is_currently_valid(&self) -> Result<bool, CertificateError> {
        let now = current_unix_time()?;
        Ok(self.is_valid_at(now))
    }

    /// Returns whether the certificate is currently expired according to system time.
    pub fn is_currently_expired(&self) -> Result<bool, CertificateError> {
        let now = current_unix_time()?;
        Ok(self.is_expired_at(now))
    }

    /// Returns whether the certificate is currently not yet valid according to system time.
    pub fn is_currently_not_yet_valid(&self) -> Result<bool, CertificateError> {
        let now = current_unix_time()?;
        Ok(self.is_not_yet_valid_at(now))
    }
}

/// Returns the current UNIX timestamp in seconds.
fn current_unix_time() -> Result<u64, CertificateError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| CertificateError::TimeError)
}

/// Validates the shared issued/expires window.
///
/// Security and correctness policy:
/// - zero timestamps are rejected,
/// - `expires_at` must be strictly greater than `issued_at`.
fn validate_validity_window(issued_at: u64, expires_at: u64) -> Result<(), CertificateError> {
    if issued_at == 0 {
        return Err(CertificateError::InvalidIssuedAt);
    }

    if expires_at == 0 {
        return Err(CertificateError::InvalidExpiresAt);
    }

    if expires_at <= issued_at {
        return Err(CertificateError::InvalidValidityWindow);
    }

    Ok(())
}

/// Validates that a canonical text field is present, non-blank, and free from
/// leading or trailing whitespace.
///
/// Error mapping policy:
/// - empty string => `empty_error`,
/// - whitespace-only string => `empty_error`,
/// - leading/trailing whitespace => `invalid_error`.
///
/// Design note:
/// `invalid_error` is borrowed rather than moved so the caller may reuse the
/// same discriminator in subsequent validation branches without ownership loss.
fn validate_canonical_text_presence<'a>(
    value: &'a str,
    empty_error: CertificateError,
    invalid_error: &CertificateError,
) -> Result<&'a str, CertificateError> {
    if value.is_empty() {
        return Err(empty_error);
    }

    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(empty_error);
    }

    if trimmed != value {
        return Err(invalid_error.clone());
    }

    Ok(trimmed)
}

/// Validates the chain field.
fn validate_chain(value: &str) -> Result<(), CertificateError> {
    let trimmed = validate_canonical_text_presence(
        value,
        CertificateError::EmptyChain,
        &CertificateError::InvalidChain,
    )?;

    if trimmed.len() > MAX_CHAIN_LEN {
        return Err(CertificateError::InvalidChain);
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(CertificateError::InvalidChain);
    }

    Ok(())
}

/// Validates the actor identifier field.
///
/// Compatibility note:
/// This validator intentionally preserves a string-based certificate-layer
/// contract instead of importing a stricter actor-id parser directly. That
/// keeps the certificate object tolerant of legacy yet bounded AOXC actor-id
/// representations while still rejecting malformed input.
fn validate_actor_id(value: &str) -> Result<(), CertificateError> {
    let trimmed = validate_canonical_text_presence(
        value,
        CertificateError::EmptyActorId,
        &CertificateError::InvalidActorId,
    )?;

    if trimmed.len() > MAX_ACTOR_ID_LEN {
        return Err(CertificateError::InvalidActorId);
    }

    if !trimmed.starts_with("AOXC-") {
        return Err(CertificateError::InvalidActorId);
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(CertificateError::InvalidActorId);
    }

    Ok(())
}

/// Validates the role field.
fn validate_role(value: &str) -> Result<(), CertificateError> {
    let trimmed = validate_canonical_text_presence(
        value,
        CertificateError::EmptyRole,
        &CertificateError::InvalidRole,
    )?;

    if trimmed.len() > MAX_ROLE_LEN {
        return Err(CertificateError::InvalidRole);
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(CertificateError::InvalidRole);
    }

    Ok(())
}

/// Validates the zone field.
fn validate_zone(value: &str) -> Result<(), CertificateError> {
    let trimmed = validate_canonical_text_presence(
        value,
        CertificateError::EmptyZone,
        &CertificateError::InvalidZone,
    )?;

    if trimmed.len() > MAX_ZONE_LEN {
        return Err(CertificateError::InvalidZone);
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(CertificateError::InvalidZone);
    }

    Ok(())
}

/// Validates the issuer field.
fn validate_issuer(value: &str) -> Result<(), CertificateError> {
    let trimmed = validate_canonical_text_presence(
        value,
        CertificateError::EmptyIssuer,
        &CertificateError::InvalidIssuer,
    )?;

    if trimmed.len() > MAX_ISSUER_LEN {
        return Err(CertificateError::InvalidIssuer);
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(CertificateError::InvalidIssuer);
    }

    Ok(())
}

/// Validates a bounded hexadecimal field with canonical text rules.
///
/// Validation policy:
/// - value must first satisfy canonical text presence checks,
/// - maximum length is enforced,
/// - odd-length hex is rejected,
/// - only ASCII hexadecimal characters are accepted.
fn validate_hex_field(
    value: &str,
    empty_error: CertificateError,
    invalid_error: CertificateError,
    max_len: usize,
) -> Result<(), CertificateError> {
    let trimmed = validate_canonical_text_presence(value, empty_error, &invalid_error)?;

    if trimmed.len() > max_len {
        return Err(invalid_error);
    }

    if trimmed.len() % 2 != 0 {
        return Err(invalid_error);
    }

    if !trimmed.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(invalid_error);
    }

    Ok(())
}

/// Validates the public-key hex field.
fn validate_pubkey_hex(value: &str) -> Result<(), CertificateError> {
    validate_hex_field(
        value,
        CertificateError::EmptyPublicKey,
        CertificateError::InvalidPublicKeyHex,
        MAX_PUBKEY_HEX_LEN,
    )
}

/// Validates the signature hex field.
fn validate_signature_hex(value: &str) -> Result<(), CertificateError> {
    validate_hex_field(
        value,
        CertificateError::EmptySignature,
        CertificateError::InvalidSignatureHex,
        MAX_SIGNATURE_HEX_LEN,
    )
}

