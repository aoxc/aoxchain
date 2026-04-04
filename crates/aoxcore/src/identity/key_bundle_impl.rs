impl NodeKeyBundleV1 {
    /// Builds a canonical key bundle from a key engine and encrypted root seed.
    ///
    /// The generated bundle:
    /// - derives deterministic role-specific Ed25519 keypairs,
    /// - stores only public metadata for each role,
    /// - preserves encrypted custody of the root seed through the supplied envelope.
    pub fn generate(
        node_name: &str,
        profile: &str,
        created_at: String,
        crypto_profile: CryptoProfile,
        engine: &KeyEngine,
        encrypted_root_seed: KeyfileEnvelope,
    ) -> Result<Self, NodeKeyBundleError> {
        let normalized_profile = normalize_profile(profile)?;

        let mut keys = NodeKeyRole::all()
            .into_iter()
            .map(|role| build_record(engine, normalized_profile, &crypto_profile, role))
            .collect::<Result<Vec<_>, _>>()?;

        keys.sort_by_key(|record| record.role);

        let mut bundle = Self {
            version: NODE_KEY_BUNDLE_VERSION,
            node_name: node_name.to_string(),
            profile: normalized_profile.to_string(),
            created_at,
            crypto_profile,
            custody_model: AOXC_NODE_KEY_CUSTODY_MODEL.to_string(),
            engine_fingerprint: engine.fingerprint(),
            bundle_fingerprint: String::new(),
            encrypted_root_seed,
            keys,
        };

        bundle.bundle_fingerprint = bundle.compute_bundle_fingerprint()?;
        bundle.validate()?;
        Ok(bundle)
    }

    /// Validates the bundle shape, role completeness, key metadata, and fingerprint integrity.
    pub fn validate(&self) -> Result<(), NodeKeyBundleError> {
        if self.version != NODE_KEY_BUNDLE_VERSION {
            return Err(NodeKeyBundleError::InvalidVersion);
        }

        if self.node_name.trim().is_empty() {
            return Err(NodeKeyBundleError::EmptyNodeName);
        }

        if self.profile.trim().is_empty() {
            return Err(NodeKeyBundleError::EmptyProfile);
        }

        if self.created_at.trim().is_empty() {
            return Err(NodeKeyBundleError::EmptyCreatedAt);
        }

        if self.custody_model.trim().is_empty() {
            return Err(NodeKeyBundleError::EmptyCustodyModel);
        }

        if self.custody_model != AOXC_NODE_KEY_CUSTODY_MODEL {
            return Err(NodeKeyBundleError::EmptyCustodyModel);
        }

        if self.engine_fingerprint.trim().is_empty() {
            return Err(NodeKeyBundleError::EmptyEngineFingerprint);
        }

        if !is_uppercase_hex_with_len(&self.engine_fingerprint, ENGINE_FINGERPRINT_HEX_LEN) {
            return Err(NodeKeyBundleError::InvalidEngineFingerprint);
        }

        if self.bundle_fingerprint.trim().is_empty() {
            return Err(NodeKeyBundleError::EmptyBundleFingerprint);
        }

        if !is_uppercase_hex_with_len(&self.bundle_fingerprint, BUNDLE_FINGERPRINT_HEX_LEN) {
            return Err(NodeKeyBundleError::InvalidBundleFingerprint);
        }

        if self.keys.is_empty() {
            return Err(NodeKeyBundleError::MissingKeys);
        }

        let normalized_profile = normalize_profile(&self.profile)?;
        if normalized_profile != self.profile {
            return Err(NodeKeyBundleError::UnsupportedProfile(self.profile.clone()));
        }

        validate_envelope(&self.encrypted_root_seed)?;

        let mut seen_roles = BTreeSet::new();

        for record in &self.keys {
            if !seen_roles.insert(record.role) {
                return Err(NodeKeyBundleError::DuplicateRole(record.role));
            }

            if record.hd_path.trim().is_empty() {
                return Err(NodeKeyBundleError::EmptyHdPath(record.role));
            }

            if record.algorithm != self.crypto_profile.operational_public_key_algorithm() {
                return Err(NodeKeyBundleError::InvalidAlgorithm(record.role));
            }

            if record.public_key_encoding != AOXC_PUBLIC_KEY_ENCODING {
                return Err(NodeKeyBundleError::InvalidPublicKeyEncoding(record.role));
            }

            if record.public_key.trim().is_empty() {
                return Err(NodeKeyBundleError::EmptyPublicKey(record.role));
            }

            if record.fingerprint.trim().is_empty() {
                return Err(NodeKeyBundleError::EmptyFingerprint(record.role));
            }

            if !record
                .public_key
                .chars()
                .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_lowercase())
            {
                return Err(NodeKeyBundleError::InvalidPublicKeyHex(record.role));
            }

            let decoded = hex::decode(&record.public_key)
                .map_err(|_| NodeKeyBundleError::InvalidPublicKeyHex(record.role))?;

            if decoded.len() != AOXC_ED25519_PUBLIC_KEY_LEN {
                return Err(NodeKeyBundleError::InvalidPublicKeyLength {
                    role: record.role,
                    expected: AOXC_ED25519_PUBLIC_KEY_LEN,
                    actual: decoded.len(),
                });
            }

            let public_key_bytes: [u8; AOXC_ED25519_PUBLIC_KEY_LEN] = decoded
                .as_slice()
                .try_into()
                .map_err(|_| NodeKeyBundleError::InvalidPublicKeyLength {
                    role: record.role,
                    expected: AOXC_ED25519_PUBLIC_KEY_LEN,
                    actual: decoded.len(),
                })?;

            let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
                .map_err(|_| NodeKeyBundleError::InvalidPublicKeyMaterial(record.role))?;

            let expected_fingerprint = fingerprint_record(&verifying_key);
            if record.fingerprint != expected_fingerprint {
                return Err(NodeKeyBundleError::FingerprintMismatch(record.role));
            }

            let parsed_path: HdPath = record.hd_path.parse()?;
            let expected_path = derive_role_path(&self.profile, record.role)?;

            if parsed_path != expected_path {
                return Err(NodeKeyBundleError::HdPathMismatch {
                    role: record.role,
                    expected: expected_path.to_string(),
                    actual: parsed_path.to_string(),
                });
            }
        }

        for role in NodeKeyRole::all() {
            if !seen_roles.contains(&role) {
                return Err(NodeKeyBundleError::MissingRole(role));
            }
        }

        let expected_bundle_fingerprint = self.compute_bundle_fingerprint()?;
        if self.bundle_fingerprint != expected_bundle_fingerprint {
            return Err(NodeKeyBundleError::BundleFingerprintMismatch);
        }

        Ok(())
    }

    /// Serializes the bundle to pretty JSON.
    pub fn to_json(&self) -> Result<String, NodeKeyBundleError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| NodeKeyBundleError::SerializationFailed(error.to_string()))
    }

    /// Deserializes the bundle from JSON and validates it.
    pub fn from_json(data: &str) -> Result<Self, NodeKeyBundleError> {
        let bundle: Self = serde_json::from_str(data)
            .map_err(|error| NodeKeyBundleError::SerializationFailed(error.to_string()))?;
        bundle.validate()?;
        Ok(bundle)
    }

    /// Returns the raw Ed25519 public-key bytes for the requested role.
    pub fn public_key_bytes_for_role(
        &self,
        role: NodeKeyRole,
    ) -> Result<[u8; AOXC_ED25519_PUBLIC_KEY_LEN], NodeKeyBundleError> {
        let record = self
            .keys
            .iter()
            .find(|record| record.role == role)
            .ok_or(NodeKeyBundleError::MissingRole(role))?;

        let bytes = hex::decode(&record.public_key)
            .map_err(|_| NodeKeyBundleError::InvalidPublicKeyHex(role))?;

        if bytes.len() != AOXC_ED25519_PUBLIC_KEY_LEN {
            return Err(NodeKeyBundleError::InvalidPublicKeyLength {
                role,
                expected: AOXC_ED25519_PUBLIC_KEY_LEN,
                actual: bytes.len(),
            });
        }

        let mut out = [0u8; AOXC_ED25519_PUBLIC_KEY_LEN];
        out.copy_from_slice(&bytes);
        Ok(out)
    }

    /// Computes the canonical bundle fingerprint.
    ///
    /// Fingerprint policy:
    /// - binds public and custody metadata,
    /// - excludes the stored `bundle_fingerprint` field itself,
    /// - keeps deterministic ordering as serialized in `keys`.
    fn compute_bundle_fingerprint(&self) -> Result<String, NodeKeyBundleError> {
        let canonical = serde_json::json!({
            "version": self.version,
            "node_name": self.node_name,
            "profile": self.profile,
            "created_at": self.created_at,
            "crypto_profile": self.crypto_profile.as_str(),
            "custody_model": self.custody_model,
            "engine_fingerprint": self.engine_fingerprint,
            "encrypted_root_seed": self.encrypted_root_seed,
            "keys": self.keys,
        });

        let bytes = serde_json::to_vec(&canonical)
            .map_err(|error| NodeKeyBundleError::SerializationFailed(error.to_string()))?;

        let mut hasher = Sha3_256::new();
        hasher.update(AOXC_NODE_BUNDLE_FINGERPRINT_DOMAIN);
        hasher.update([0x00]);
        hasher.update(bytes);

        let digest = hasher.finalize();
        Ok(hex::encode_upper(&digest[..16]))
    }
}

/// Normalizes accepted profile aliases into canonical AOXC profile names.
///
/// Current canonical names:
/// - `mainnet`
/// - `testnet`
/// - `validation`
/// - `devnet`
/// - `localnet`
///
/// Backward-compatible aliases:
/// - `validator` => `validation`
fn normalize_profile(profile: &str) -> Result<&'static str, NodeKeyBundleError> {
    match profile.trim().to_ascii_lowercase().as_str() {
        "mainnet" => Ok("mainnet"),
        "testnet" => Ok("testnet"),
        "validation" => Ok("validation"),
        "validator" => Ok("validation"),
        "devnet" => Ok("devnet"),
        "localnet" => Ok("localnet"),
        other => Err(NodeKeyBundleError::UnsupportedProfile(other.to_string())),
    }
}

/// Derives the canonical AOXC HD path for the requested operational profile and role.
///
/// Important compatibility note:
/// these chain identifiers are intentionally kept within the canonical unhardened
/// 31-bit HD component range so they remain compatible with strict `HdPath` validation.
fn derive_role_path(profile: &str, role: NodeKeyRole) -> Result<HdPath, NodeKeyBundleError> {
    let normalized = normalize_profile(profile)?;

    let chain = match normalized {
        "mainnet" => 26_260_001,
        "testnet" => 26_260_101,
        "validation" => 26_260_301,
        "devnet" => 26_260_201,
        "localnet" => 26_269_001,
        _ => {
            return Err(NodeKeyBundleError::UnsupportedProfile(
                normalized.to_string(),
            ));
        }
    };

    Ok(HdPath::new(chain, role.role_index(), 1, 0)?)
}

/// Builds a canonical public node-key record for the requested role.
fn build_record(
    engine: &KeyEngine,
    profile: &str,
    crypto_profile: &CryptoProfile,
    role: NodeKeyRole,
) -> Result<NodeKeyRecord, NodeKeyBundleError> {
    let path = derive_role_path(profile, role)?;
    let material = engine.derive_key_material(&path)?;
    let signing_key = derive_ed25519_signing_key(&material, role);
    let verifying_key: VerifyingKey = signing_key.verifying_key();

    Ok(NodeKeyRecord {
        role,
        hd_path: path.to_string(),
        algorithm: crypto_profile
            .operational_public_key_algorithm()
            .to_string(),
        public_key_encoding: AOXC_PUBLIC_KEY_ENCODING.to_string(),
        public_key: hex::encode_upper(verifying_key.to_bytes()),
        fingerprint: fingerprint_record(&verifying_key),
    })
}

/// Derives a deterministic Ed25519 signing key from canonical AOXC role material.
///
/// The derivation process intentionally avoids reusing the 64-byte engine output
/// directly as an Ed25519 expanded secret. Instead, it compresses the role-scoped
/// material through a dedicated domain-separated hash and uses the first 32 bytes
/// as the Ed25519 seed material.
fn derive_ed25519_signing_key(
    material: &[u8; DERIVED_ENTROPY_LEN],
    role: NodeKeyRole,
) -> SigningKey {
    let mut hasher = Sha3_256::new();
    hasher.update(AOXC_NODE_BUNDLE_ED25519_ROLE_SEED_DOMAIN);
    hasher.update([0x00]);
    hasher.update(role.as_str().as_bytes());
    hasher.update([0x00]);
    hasher.update(material);

    let digest = hasher.finalize();
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&digest[..32]);

    SigningKey::from_bytes(&seed)
}

/// Derives a stable short fingerprint from an Ed25519 verifying key.
fn fingerprint_record(public_key: &VerifyingKey) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(AOXC_NODE_BUNDLE_PUBLIC_KEY_FINGERPRINT_DOMAIN);
    hasher.update([0x00]);
    hasher.update(public_key.to_bytes());

    let digest = hasher.finalize();
    hex::encode_upper(&digest[..8])
}

/// Returns whether the provided string is uppercase hexadecimal of an exact length.
fn is_uppercase_hex_with_len(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_lowercase())
}

