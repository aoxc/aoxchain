use super::super::*;

pub(in crate::cli::bootstrap::operations) fn bootstrap_profile_directory(
    output_dir: &Path,
    profile: EnvironmentProfile,
    operator_name: &str,
    password: &str,
) -> Result<ProfileBootstrapSummary, AppError> {
    fs::create_dir_all(output_dir).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!(
                "Failed to create bootstrap output directory {}",
                output_dir.display()
            ),
            error,
        )
    })?;

    let home_dir = output_dir.join("home");
    ensure_layout(&home_dir)?;

    let _home_override = ScopedHomeOverride::install(&home_dir);

    let settings = build_profile_settings(home_dir.display().to_string(), profile, None)?;
    persist(&settings)?;

    let mut genesis = profile.genesis_document();
    let material = bootstrap_operator_key(operator_name, profile.as_str(), password)?;
    let operator_summary = inspect_operator_key()?;
    upsert_validator_account(
        &mut genesis,
        &operator_summary,
        DEFAULT_VALIDATOR_GENESIS_BALANCE,
    )?;
    materialize_binding_documents(&genesis, &operator_summary, &settings)?;

    let genesis_json = serde_json::to_string_pretty(&genesis).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode bootstrap genesis",
            error,
        )
    })?;
    write_file(&genesis_path()?, &genesis_json)?;

    let operator_fp = operator_fingerprint()?;
    let consensus_pk = consensus_public_key_hex()?;
    let node_state = bootstrap_state()?;

    let material_path = home_dir.join("support").join("operator-key-bootstrap.json");
    let material_json = serde_json::to_string_pretty(&material).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode operator bootstrap material",
            error,
        )
    })?;
    write_file(&material_path, &material_json)?;

    Ok(ProfileBootstrapSummary {
        profile: profile.as_str().to_string(),
        home_dir: home_dir.display().to_string(),
        bind_host: settings.network.bind_host.clone(),
        p2p_port: settings.network.p2p_port,
        rpc_port: settings.network.rpc_port,
        prometheus_port: settings.telemetry.prometheus_port,
        chain_id: genesis.identity.chain_id,
        network_id: genesis.identity.network_id,
        operator_fingerprint: operator_fp,
        consensus_public_key: consensus_pk,
        node_height: node_state.current_height,
    })
}

pub(in crate::cli::bootstrap::operations) fn build_profile_settings(
    home_dir: String,
    profile: EnvironmentProfile,
    bind_host: Option<String>,
) -> Result<Settings, AppError> {
    let mut settings = Settings::default_for_profile(home_dir, profile.as_str())
        .map_err(|error| AppError::new(ErrorCode::ConfigInvalid, error))?;

    if let Some(bind_host) = bind_host {
        settings.network.bind_host = bind_host;
    }

    settings
        .validate()
        .map_err(|error| AppError::new(ErrorCode::ConfigInvalid, error))?;

    Ok(settings)
}

pub(in crate::cli::bootstrap::operations) fn upsert_validator_account(
    genesis: &mut BootstrapGenesisDocument,
    operator: &crate::keys::material::KeyMaterialSummary,
    balance: &str,
) -> Result<(), AppError> {
    if !is_non_zero_decimal_string(balance) {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Validator genesis balance must be a non-zero decimal string",
        ));
    }

    let account_id = format!(
        "AOXC_VALIDATOR_{}",
        operator
            .bundle_fingerprint
            .chars()
            .take(16)
            .collect::<String>()
    );

    if let Some(existing) = genesis
        .state
        .accounts
        .iter_mut()
        .find(|record| record.account_id == account_id)
    {
        existing.role = "validator".to_string();
        existing.balance = balance.to_string();
    } else {
        genesis.state.accounts.push(BootstrapAccountRecord {
            account_id,
            balance: balance.to_string(),
            role: "validator".to_string(),
        });
    }

    Ok(())
}

pub(in crate::cli::bootstrap::operations) fn materialize_binding_documents(
    genesis: &BootstrapGenesisDocument,
    operator: &crate::keys::material::KeyMaterialSummary,
    settings: &Settings,
) -> Result<(), AppError> {
    let identity_dir = genesis_path()?
        .parent()
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::FilesystemIoFailed,
                "Failed to resolve identity directory for genesis bindings",
            )
        })?
        .to_path_buf();

    let short_fp = operator
        .bundle_fingerprint
        .chars()
        .take(12)
        .collect::<String>()
        .to_ascii_lowercase();
    let validator_id = format!("aoxc-val-{}-{short_fp}", genesis.environment);
    let bootnode_id = format!("aoxc-boot-{}-{short_fp}", genesis.environment);
    let display_name = format!(
        "AOXC {} validator {}",
        genesis.environment.to_ascii_uppercase(),
        &short_fp
    );

    let validators_doc = BootstrapValidatorBindingsDocument {
        schema_version: 2,
        environment: genesis.environment.clone(),
        identity: genesis.identity.clone(),
        validators: vec![BootstrapValidatorBindingRecord {
            validator_id: validator_id.clone(),
            display_name: display_name.clone(),
            role: "validator".to_string(),
            consensus_key_algorithm: "ed25519".to_string(),
            consensus_public_key_encoding: "hex".to_string(),
            consensus_public_key: operator.consensus_public_key.clone(),
            consensus_key_fingerprint: operator.consensus_key_fingerprint.clone(),
            network_key_algorithm: "ed25519".to_string(),
            network_public_key_encoding: "hex".to_string(),
            network_public_key: operator.transport_public_key.clone(),
            network_key_fingerprint: operator.transport_key_fingerprint.clone(),
            weight: 1,
            status: "active".to_string(),
        }],
    };
    write_json_pretty(
        &identity_dir.join(&genesis.bindings.validators_file),
        &validators_doc,
        "Failed to encode validators binding document",
    )?;

    let bootnodes_doc = BootstrapBootnodesDocument {
        schema_version: 2,
        environment: genesis.environment.clone(),
        identity: genesis.identity.clone(),
        bootnodes: vec![BootstrapBootnodeRecord {
            node_id: bootnode_id,
            display_name: format!("AOXC {} bootnode {}", genesis.environment, short_fp),
            transport_key_algorithm: "ed25519".to_string(),
            transport_public_key_encoding: "hex".to_string(),
            transport_public_key: operator.transport_public_key.clone(),
            transport_key_fingerprint: operator.transport_key_fingerprint.clone(),
            address: format!(
                "{}:{}",
                settings.network.bind_host, settings.network.p2p_port
            ),
            transport: "tcp".to_string(),
            status: "active".to_string(),
        }],
    };
    write_json_pretty(
        &identity_dir.join(&genesis.bindings.bootnodes_file),
        &bootnodes_doc,
        "Failed to encode bootnodes binding document",
    )?;

    let issued_at = chrono::Utc::now().to_rfc3339();
    let fingerprint_seed = format!(
        "{}:{}:{}:{}",
        genesis.identity.network_id, validator_id, operator.bundle_fingerprint, issued_at
    );
    let cert_fingerprint = hex::encode(Sha256::digest(fingerprint_seed.as_bytes()));
    let certificate_doc = BootstrapCertificateDocument {
        schema_version: 1,
        certificate_kind: "aoxc-environment-certificate".to_string(),
        environment: genesis.environment.clone(),
        identity: genesis.identity.clone(),
        certificate: BootstrapCertificateBody {
            status: "active".to_string(),
            issuer: "AOXC Bootstrap Authority".to_string(),
            subject: format!("{} Environment Bundle", genesis.identity.chain_name),
            certificate_serial: format!(
                "AOXC-CERT-{}-{}",
                genesis.environment.to_ascii_uppercase(),
                genesis.identity.network_serial
            ),
            issued_at,
            expires_at: None,
            fingerprint_sha256: cert_fingerprint,
            signature_algorithm: "ed25519".to_string(),
            signature: operator.consensus_key_fingerprint.clone(),
        },
        metadata: BootstrapMetadata {
            description: format!(
                "Generated AOXC bootstrap certificate for {}",
                genesis.environment
            ),
            status: "active".to_string(),
        },
    };
    write_json_pretty(
        &identity_dir.join(&genesis.bindings.certificate_file),
        &certificate_doc,
        "Failed to encode certificate binding document",
    )?;

    if let Some(accounts_file) = &genesis.bindings.accounts_file {
        #[derive(Serialize)]
        struct AccountsDoc<'a> {
            schema_version: u8,
            environment: &'a str,
            identity: &'a CanonicalIdentity,
            accounts: &'a [BootstrapAccountRecord],
        }

        let accounts_doc = AccountsDoc {
            schema_version: 1,
            environment: &genesis.environment,
            identity: &genesis.identity,
            accounts: &genesis.state.accounts,
        };

        write_json_pretty(
            &identity_dir.join(accounts_file),
            &accounts_doc,
            "Failed to encode accounts binding document",
        )?;
    }

    Ok(())
}

pub(in crate::cli::bootstrap::operations) fn write_json_pretty<T: Serialize>(
    path: &Path,
    payload: &T,
    context: &str,
) -> Result<(), AppError> {
    let encoded = serde_json::to_string_pretty(payload)
        .map_err(|error| AppError::with_source(ErrorCode::OutputEncodingFailed, context, error))?;
    write_file(path, &encoded)
}

pub(in crate::cli::bootstrap::operations) fn load_genesis()
-> Result<BootstrapGenesisDocument, AppError> {
    let raw = read_file(&genesis_path()?)?;
    serde_json::from_str::<BootstrapGenesisDocument>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            "Failed to decode AOXC genesis document",
            error,
        )
    })
}

pub(in crate::cli::bootstrap::operations) fn persist_genesis(
    genesis: &BootstrapGenesisDocument,
) -> Result<(), AppError> {
    write_json_pretty(
        &genesis_path()?,
        genesis,
        "Failed to encode AOXC genesis document",
    )
}

pub(in crate::cli::bootstrap::operations) fn identity_dir_from_genesis(
    genesis: &BootstrapGenesisDocument,
) -> Result<PathBuf, AppError> {
    let path = genesis_path()?;
    let root = path.parent().ok_or_else(|| {
        AppError::new(
            ErrorCode::FilesystemIoFailed,
            "Failed to resolve identity directory for genesis bindings",
        )
    })?;

    if genesis.bindings.validators_file.trim().is_empty()
        || genesis.bindings.bootnodes_file.trim().is_empty()
        || genesis.bindings.certificate_file.trim().is_empty()
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis binding file references must not be empty",
        ));
    }

    Ok(root.to_path_buf())
}

pub(in crate::cli::bootstrap::operations) fn sync_optional_accounts_binding(
    genesis: &BootstrapGenesisDocument,
) -> Result<(), AppError> {
    let Some(accounts_file) = genesis.bindings.accounts_file.as_deref() else {
        return Ok(());
    };

    #[derive(Serialize)]
    struct AccountsDoc<'a> {
        schema_version: u8,
        environment: &'a str,
        identity: &'a CanonicalIdentity,
        accounts: &'a [BootstrapAccountRecord],
    }

    let doc = AccountsDoc {
        schema_version: 1,
        environment: &genesis.environment,
        identity: &genesis.identity,
        accounts: &genesis.state.accounts,
    };

    let path = identity_dir_from_genesis(genesis)?.join(accounts_file);
    write_json_pretty(&path, &doc, "Failed to encode accounts binding document")
}

pub(in crate::cli::bootstrap::operations) fn derive_short_fingerprint(value: &str) -> String {
    let digest = Sha256::digest(value.trim().as_bytes());
    hex::encode(digest)[..16].to_string()
}

pub(in crate::cli::bootstrap::operations) fn load_or_default_validators_binding(
    genesis: &BootstrapGenesisDocument,
) -> Result<BootstrapValidatorBindingsDocument, AppError> {
    let path = identity_dir_from_genesis(genesis)?.join(&genesis.bindings.validators_file);
    if !path.exists() {
        return Ok(BootstrapValidatorBindingsDocument {
            schema_version: 2,
            environment: genesis.environment.clone(),
            identity: genesis.identity.clone(),
            validators: Vec::new(),
        });
    }

    let raw = read_file(&path)?;
    serde_json::from_str::<BootstrapValidatorBindingsDocument>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            format!(
                "Failed to decode validators binding document: {}",
                path.display()
            ),
            error,
        )
    })
}

pub(in crate::cli::bootstrap::operations) fn upsert_validator_binding(
    doc: &mut BootstrapValidatorBindingsDocument,
    record: BootstrapValidatorBindingRecord,
) {
    if let Some(existing) = doc
        .validators
        .iter_mut()
        .find(|existing| existing.validator_id == record.validator_id)
    {
        *existing = record;
    } else {
        doc.validators.push(record);
    }
}

pub(in crate::cli::bootstrap::operations) fn persist_validators_binding(
    genesis: &BootstrapGenesisDocument,
    doc: &BootstrapValidatorBindingsDocument,
) -> Result<(), AppError> {
    let path = identity_dir_from_genesis(genesis)?.join(&genesis.bindings.validators_file);
    write_json_pretty(&path, doc, "Failed to encode validators binding document")
}

pub(in crate::cli::bootstrap::operations) fn load_or_default_bootnodes_binding(
    genesis: &BootstrapGenesisDocument,
) -> Result<BootstrapBootnodesDocument, AppError> {
    let path = identity_dir_from_genesis(genesis)?.join(&genesis.bindings.bootnodes_file);
    if !path.exists() {
        return Ok(BootstrapBootnodesDocument {
            schema_version: 2,
            environment: genesis.environment.clone(),
            identity: genesis.identity.clone(),
            bootnodes: Vec::new(),
        });
    }

    let raw = read_file(&path)?;
    serde_json::from_str::<BootstrapBootnodesDocument>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            format!(
                "Failed to decode bootnodes binding document: {}",
                path.display()
            ),
            error,
        )
    })
}

pub(in crate::cli::bootstrap::operations) fn upsert_bootnode_binding(
    doc: &mut BootstrapBootnodesDocument,
    record: BootstrapBootnodeRecord,
) {
    if let Some(existing) = doc
        .bootnodes
        .iter_mut()
        .find(|existing| existing.node_id == record.node_id)
    {
        *existing = record;
    } else {
        doc.bootnodes.push(record);
    }
}

pub(in crate::cli::bootstrap::operations) fn persist_bootnodes_binding(
    genesis: &BootstrapGenesisDocument,
    doc: &BootstrapBootnodesDocument,
) -> Result<(), AppError> {
    let path = identity_dir_from_genesis(genesis)?.join(&genesis.bindings.bootnodes_file);
    write_json_pretty(&path, doc, "Failed to encode bootnodes binding document")
}

pub(in crate::cli::bootstrap::operations) fn validate_genesis(
    genesis: &BootstrapGenesisDocument,
) -> Result<(), AppError> {
    const ALLOWED_ACCOUNT_ROLES: [&str; 12] = [
        "treasury",
        "validator",
        "system",
        "user",
        "governance",
        "forge",
        "quorum",
        "seal",
        "archive",
        "sentinel",
        "relay",
        "pocket",
    ];

    if genesis.schema_version != 1 {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: schema_version must be 1",
        ));
    }

    if genesis.genesis_kind.trim() != "aoxc-genesis-config" {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: genesis_kind mismatch",
        ));
    }

    if genesis.environment.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: environment must not be empty",
        ));
    }

    if genesis.family_name.trim().is_empty() || genesis.family_code.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: family identity fields must not be empty",
        ));
    }

    if genesis.identity.family_id != 2626 {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: family_id must equal 2626",
        ));
    }

    if genesis.identity.chain_name.trim().is_empty()
        || genesis.identity.network_class.trim().is_empty()
        || genesis.identity.network_serial.trim().is_empty()
        || genesis.identity.network_id.trim().is_empty()
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: identity fields must not be empty",
        ));
    }

    if genesis.identity.chain_id == 0 {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: chain_id must be non-zero",
        ));
    }

    if genesis.environment.trim() != genesis.identity.network_class.trim()
        && !matches!(
            (
                genesis.environment.trim(),
                genesis.identity.network_class.trim()
            ),
            ("mainnet", "public_mainnet") | ("testnet", "public_testnet")
        )
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: environment and network_class are inconsistent",
        ));
    }

    if genesis.consensus.block_time_ms == 0 {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: block_time_ms must be non-zero",
        ));
    }

    if genesis.vm.vm_engine.trim().is_empty() || genesis.vm.gas_model.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: vm engine and gas model must not be empty",
        ));
    }

    if genesis.vm.block_gas_limit == 0 || genesis.vm.tx_gas_limit == 0 {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: VM gas limits must be non-zero",
        ));
    }

    if genesis.vm.tx_gas_limit > genesis.vm.block_gas_limit {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: tx_gas_limit cannot exceed block_gas_limit",
        ));
    }

    if !is_non_zero_decimal_string(&genesis.vm.min_gas_price) {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: min_gas_price must be a non-zero decimal string",
        ));
    }

    if genesis.vm.max_contract_size_bytes == 0 || genesis.vm.max_call_depth == 0 {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: VM execution bounds must be non-zero",
        ));
    }

    if genesis.economics.native_symbol.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: native_symbol must not be empty",
        ));
    }

    if !is_non_zero_decimal_string(&genesis.economics.initial_treasury.amount) {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: treasury amount must be a non-zero decimal string",
        ));
    }

    if genesis.state.accounts.is_empty() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: at least one state account is required",
        ));
    }

    let mut seen_accounts = BTreeSet::new();
    for account in &genesis.state.accounts {
        if account.account_id.trim().is_empty()
            || account.role.trim().is_empty()
            || !is_decimal_string(&account.balance)
        {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                "Genesis validation failed: account fields are invalid",
            ));
        }

        if !seen_accounts.insert(account.account_id.clone()) {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                "Genesis validation failed: duplicate account_id detected",
            ));
        }

        if !ALLOWED_ACCOUNT_ROLES.contains(&account.role.trim().to_ascii_lowercase().as_str()) {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                format!(
                    "Genesis validation failed: unsupported account role '{}' for {}",
                    account.role, account.account_id
                ),
            ));
        }
    }

    if genesis.bindings.validators_file.trim().is_empty()
        || genesis.bindings.bootnodes_file.trim().is_empty()
        || genesis.bindings.certificate_file.trim().is_empty()
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: binding file references must not be empty",
        ));
    }

    if genesis.integrity.hash_algorithm.trim() != "sha256" {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: hash_algorithm must equal sha256",
        ));
    }

    if !genesis.integrity.deterministic_serialization_required {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            "Genesis validation failed: deterministic serialization must be required",
        ));
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct ValidatorSetDocument {
    validators: Vec<ValidatorRecord>,
}

#[derive(Debug, Deserialize)]
struct ValidatorRecord {
    validator_id: String,
    consensus_public_key: String,
    status: String,
}

pub(in crate::cli::bootstrap::operations) fn validate_binding_files(
    genesis: &BootstrapGenesisDocument,
) -> Result<(), AppError> {
    let genesis_file = genesis_path()?;
    let root = genesis_file.parent().ok_or_else(|| {
        AppError::new(
            ErrorCode::FilesystemIoFailed,
            "Genesis validation failed: identity directory is not accessible",
        )
    })?;

    let validators_path = root.join(&genesis.bindings.validators_file);
    let validators_raw = read_file(&validators_path)?;
    let validators_doc: ValidatorSetDocument =
        serde_json::from_str(&validators_raw).map_err(|error| {
            AppError::with_source(
                ErrorCode::ConfigInvalid,
                format!(
                    "Genesis validation failed: validators binding is not valid JSON: {}",
                    validators_path.display()
                ),
                error,
            )
        })?;

    if validators_doc.validators.is_empty() {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "Genesis validation failed: validators file must contain at least one validator: {}",
                validators_path.display()
            ),
        ));
    }

    if matches!(genesis.environment.as_str(), "mainnet" | "testnet") {
        let minimum_validators = if genesis.environment == "mainnet" {
            4
        } else {
            3
        };
        if validators_doc.validators.len() < minimum_validators {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                format!(
                    "Genesis validation failed: {} requires at least {} validators",
                    genesis.environment, minimum_validators
                ),
            ));
        }
    }

    for validator in &validators_doc.validators {
        if validator.validator_id.trim().is_empty() {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                "Genesis validation failed: validator_id must not be empty",
            ));
        }

        if validator.consensus_public_key.trim().is_empty()
            || validator
                .consensus_public_key
                .to_ascii_lowercase()
                .contains("pending_real_value")
        {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                format!(
                    "Genesis validation failed: validator consensus key is empty or placeholder for {}",
                    validator.validator_id
                ),
            ));
        }

        if validator.status.trim() != "active" {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                format!(
                    "Genesis validation failed: validator {} is not active",
                    validator.validator_id
                ),
            ));
        }
    }

    let bootnodes_path = root.join(&genesis.bindings.bootnodes_file);
    let bootnodes_raw = read_file(&bootnodes_path)?;
    let bootnodes_json: Value = serde_json::from_str(&bootnodes_raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            format!(
                "Genesis validation failed: bootnodes binding is not valid JSON: {}",
                bootnodes_path.display()
            ),
            error,
        )
    })?;

    if bootnodes_json
        .get("bootnodes")
        .and_then(Value::as_array)
        .is_none_or(|entries| entries.is_empty())
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "Genesis validation failed: bootnodes file must contain at least one bootnode: {}",
                bootnodes_path.display()
            ),
        ));
    }

    if matches!(genesis.environment.as_str(), "mainnet" | "testnet") {
        let minimum_bootnodes = if genesis.environment == "mainnet" {
            3
        } else {
            2
        };
        let bootnode_count = bootnodes_json
            .get("bootnodes")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        if bootnode_count < minimum_bootnodes {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                format!(
                    "Genesis validation failed: {} requires at least {} bootnodes",
                    genesis.environment, minimum_bootnodes
                ),
            ));
        }
    }

    let certificate_path = root.join(&genesis.bindings.certificate_file);
    let certificate_raw = read_file(&certificate_path)?;
    let certificate_json: Value = serde_json::from_str(&certificate_raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::ConfigInvalid,
            format!(
                "Genesis validation failed: certificate binding is not valid JSON: {}",
                certificate_path.display()
            ),
            error,
        )
    })?;

    if certificate_json
        .as_object()
        .is_none_or(|object| object.is_empty())
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "Genesis validation failed: certificate file is empty: {}",
                certificate_path.display()
            ),
        ));
    }

    if let Some(accounts_file) = &genesis.bindings.accounts_file {
        let accounts_path = root.join(accounts_file);
        let accounts_raw = read_file(&accounts_path)?;
        let accounts_json: Value = serde_json::from_str(&accounts_raw).map_err(|error| {
            AppError::with_source(
                ErrorCode::ConfigInvalid,
                format!(
                    "Genesis validation failed: accounts binding is not valid JSON: {}",
                    accounts_path.display()
                ),
                error,
            )
        })?;

        if accounts_json
            .get("accounts")
            .and_then(Value::as_array)
            .is_none_or(|entries| entries.is_empty())
        {
            return Err(AppError::new(
                ErrorCode::ConfigInvalid,
                format!(
                    "Genesis validation failed: accounts file must contain at least one account: {}",
                    accounts_path.display()
                ),
            ));
        }
    }

    Ok(())
}

pub(in crate::cli::bootstrap::operations) fn validate_identity_against_repo_policy(
    genesis: &BootstrapGenesisDocument,
) -> Result<(), AppError> {
    let repo_root = resolve_repo_root_for_configs()?;
    let profile_path = repo_root
        .join("configs")
        .join("environments")
        .join(&genesis.environment)
        .join("profile.toml");
    let release_path = repo_root
        .join("configs")
        .join("environments")
        .join(&genesis.environment)
        .join("release-policy.toml");
    let registry_path = repo_root
        .join("configs")
        .join("registry")
        .join("network-registry.toml");

    let profile = parse_identity_tuple_from_toml_section(&profile_path, "identity")?;
    let release = parse_identity_tuple_from_toml_section(&release_path, "identity")?;
    let registry = parse_registry_identity_tuple(&registry_path, &genesis.environment)?;

    ensure_identity_tuple_match(
        "genesis<->profile",
        genesis.identity.chain_id,
        &genesis.identity.network_id,
        &genesis.identity.network_serial,
        profile.chain_id,
        &profile.network_id,
        &profile.network_serial,
    )?;
    ensure_identity_tuple_match(
        "genesis<->release-policy",
        genesis.identity.chain_id,
        &genesis.identity.network_id,
        &genesis.identity.network_serial,
        release.chain_id,
        &release.network_id,
        &release.network_serial,
    )?;
    ensure_identity_tuple_match(
        "genesis<->registry",
        genesis.identity.chain_id,
        &genesis.identity.network_id,
        &genesis.identity.network_serial,
        registry.chain_id,
        &registry.network_id,
        &registry.network_serial,
    )?;

    Ok(())
}

fn resolve_repo_root_for_configs() -> Result<PathBuf, AppError> {
    let current = std::env::current_dir().map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            "Strict identity validation failed: cannot resolve current directory",
            error,
        )
    })?;

    for candidate in current.ancestors() {
        if candidate.join("Cargo.toml").exists() && candidate.join("configs").exists() {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(AppError::new(
        ErrorCode::FilesystemIoFailed,
        "Strict identity validation failed: repository root with configs/ not found",
    ))
}

#[derive(Clone)]
struct IdentityTuple {
    chain_id: u64,
    network_id: String,
    network_serial: String,
}

fn parse_identity_tuple_from_toml_section(
    path: &Path,
    section: &str,
) -> Result<IdentityTuple, AppError> {
    let raw = read_file(path)?;
    let mut in_section = false;
    let mut chain_id = None::<u64>;
    let mut network_id = None::<String>;
    let mut network_serial = None::<String>;

    for line in raw.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_section = line
                .trim_matches(|ch| ch == '[' || ch == ']')
                .trim()
                .eq(section);
            continue;
        }
        if !in_section {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "chain_id" => chain_id = value.parse::<u64>().ok(),
                "network_id" => network_id = Some(value.trim_matches('"').to_string()),
                "network_serial" => network_serial = Some(value.trim_matches('"').to_string()),
                _ => {}
            }
        }
    }

    match (chain_id, network_id, network_serial) {
        (Some(chain_id), Some(network_id), Some(network_serial)) => Ok(IdentityTuple {
            chain_id,
            network_id,
            network_serial,
        }),
        _ => Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "Strict identity validation failed: {} is missing chain_id/network_id/network_serial in [{}]",
                path.display(),
                section
            ),
        )),
    }
}

fn parse_registry_identity_tuple(
    path: &Path,
    environment: &str,
) -> Result<IdentityTuple, AppError> {
    let raw = read_file(path)?;
    let section_name = match environment {
        "mainnet" => "canonical_networks.mainnet",
        "testnet" => "canonical_networks.testnet",
        "validation" => "canonical_networks.validation",
        "localnet" => "canonical_networks.localnet",
        other => {
            return Err(AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!(
                    "Strict identity validation does not support environment '{}'",
                    other
                ),
            ));
        }
    };

    let mut in_section = false;
    let mut chain_id = None::<u64>;
    let mut network_id = None::<String>;
    let mut network_serial = None::<String>;

    for line in raw.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_section = line
                .trim_matches(|ch| ch == '[' || ch == ']')
                .trim()
                .eq(section_name);
            continue;
        }
        if !in_section {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "chain_id" => chain_id = value.parse::<u64>().ok(),
                "network_id" => network_id = Some(value.trim_matches('"').to_string()),
                "network_serial" => network_serial = Some(value.trim_matches('"').to_string()),
                _ => {}
            }
        }
    }

    match (chain_id, network_id, network_serial) {
        (Some(chain_id), Some(network_id), Some(network_serial)) => Ok(IdentityTuple {
            chain_id,
            network_id,
            network_serial,
        }),
        _ => Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "Strict identity validation failed: {} is missing identity tuple in [{}]",
                path.display(),
                section_name
            ),
        )),
    }
}

fn ensure_identity_tuple_match(
    label: &str,
    chain_id_a: u64,
    network_id_a: &str,
    network_serial_a: &str,
    chain_id_b: u64,
    network_id_b: &str,
    network_serial_b: &str,
) -> Result<(), AppError> {
    if chain_id_a != chain_id_b
        || !network_id_a.eq(network_id_b)
        || !network_serial_a.eq(network_serial_b)
    {
        return Err(AppError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "Strict identity validation failed: {} mismatch (chain_id {} vs {}, network_id {} vs {}, network_serial {} vs {})",
                label,
                chain_id_a,
                chain_id_b,
                network_id_a,
                network_id_b,
                network_serial_a,
                network_serial_b
            ),
        ));
    }

    Ok(())
}
