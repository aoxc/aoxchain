impl ContractManifestBuilder {
    /// Returns a builder with canonical AOXC defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a builder preconfigured for WASM artifacts.
    pub fn wasm() -> Self {
        Self::default().with_vm_target(VmTarget::Wasm)
    }

    /// Returns a builder preconfigured for EVM artifacts.
    pub fn evm() -> Self {
        Self::default().with_vm_target(VmTarget::Evm)
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_package(mut self, package: impl Into<String>) -> Self {
        self.package = Some(package.into());
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn with_contract_version(mut self, version: impl Into<String>) -> Self {
        self.contract_version = Some(version.into());
        self
    }

    pub fn with_vm_target(mut self, vm_target: VmTarget) -> Self {
        self.vm_target = Some(vm_target);
        self
    }

    pub fn with_artifact_digest(mut self, digest: ArtifactDigest) -> Self {
        self.artifact_digest = Some(digest);
        self
    }

    pub fn with_artifact_size(mut self, artifact_size: u64) -> Self {
        self.artifact_size = artifact_size;
        self
    }

    pub fn with_artifact_location(mut self, location: impl Into<String>) -> Self {
        self.artifact_location = Some(location.into());
        self
    }

    pub fn with_artifact_location_kind(mut self, kind: ArtifactLocationKind) -> Self {
        self.artifact_location_kind = kind;
        self
    }

    pub fn with_compression(mut self, compression: impl Into<String>) -> Self {
        self.compression = Some(compression.into());
        self
    }

    pub fn with_metadata(mut self, metadata: ContractMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn add_entrypoint(mut self, entrypoint: Entrypoint) -> Self {
        self.entrypoints.push(entrypoint);
        self
    }

    pub fn allow_capability(mut self, capability: ContractCapability) -> Self {
        self.allowed_capabilities.push(capability);
        self
    }

    pub fn forbid_capability(mut self, capability: ContractCapability) -> Self {
        self.forbidden_capabilities.push(capability);
        self
    }

    pub fn with_max_artifact_size(mut self, max_artifact_size: u64) -> Self {
        self.max_artifact_size = max_artifact_size;
        self
    }

    pub fn with_review_required(mut self, review_required: bool) -> Self {
        self.review_required = review_required;
        self
    }

    pub fn with_signature_required(mut self, signature_required: bool) -> Self {
        self.signature_required = signature_required;
        self
    }

    pub fn with_source_trust_level(mut self, source_trust_level: SourceTrustLevel) -> Self {
        self.source_trust_level = source_trust_level;
        self
    }

    pub fn with_schema_version(mut self, schema_version: u32) -> Self {
        self.schema_version = schema_version;
        self
    }

    pub fn with_minimum_schema_version(mut self, minimum_schema_version: u32) -> Self {
        self.minimum_schema_version = minimum_schema_version;
        self
    }

    pub fn with_supported_schema_versions(mut self, versions: Vec<u32>) -> Self {
        self.supported_schema_versions = versions;
        self
    }

    pub fn with_supported_runtime_families(mut self, families: Vec<RuntimeFamily>) -> Self {
        self.supported_runtime_families = families;
        self
    }

    pub fn with_supported_network_classes(mut self, classes: Vec<NetworkClass>) -> Self {
        self.supported_network_classes = classes;
        self
    }

    pub fn with_execution_profile(mut self, execution_profile: ExecutionProfile) -> Self {
        self.execution_profile = Some(execution_profile);
        // Preserve fluent-builder call-order semantics: if a full execution profile
        // is supplied after field-level overrides, the full profile becomes the
        // canonical source of truth until a later field-level override is applied.
        self.contract_class = None;
        self.capability_profile = None;
        self.policy_profile = None;
        self
    }

    pub fn with_contract_class(mut self, contract_class: ContractClass) -> Self {
        self.contract_class = Some(contract_class);
        self
    }

    pub fn with_capability_profile(mut self, capability_profile: CapabilityProfile) -> Self {
        self.capability_profile = Some(capability_profile);
        self
    }

    pub fn with_policy_profile(mut self, policy_profile: PolicyProfile) -> Self {
        self.policy_profile = Some(policy_profile);
        self
    }

    /// Builds a validated contract manifest.
    ///
    /// Validation discipline:
    /// - Mandatory builder inputs are extracted in deterministic order.
    /// - Runtime-family and network-class compatibility defaults are filled only
    ///   when the caller did not provide explicit values.
    /// - Downstream semantic validation remains delegated to `aoxcontract`.
    pub fn build(self) -> Result<ContractManifest, BuilderError> {
        let name = required(self.name, "name")?;
        let package = required(self.package, "package")?;
        let version = required(self.version, "version")?;
        let contract_version =
            ContractVersion(required(self.contract_version, "contract_version")?);
        let vm_target = required(self.vm_target, "vm_target")?;
        let digest = required(self.artifact_digest, "artifact_digest")?;
        let artifact_location = required(self.artifact_location, "artifact_location")?;

        let artifact_format = artifact_format_for_vm(&vm_target);
        let artifact_media_type = default_media_type_for_format(&artifact_format).to_string();

        let supported_runtime_families =
            normalize_supported_runtime_families(self.supported_runtime_families, &vm_target);

        let supported_network_classes =
            normalize_supported_network_classes(self.supported_network_classes);

        let artifact = ContractArtifactRef::new(
            digest.clone(),
            self.artifact_size,
            artifact_format.clone(),
            self.artifact_location_kind,
            artifact_location,
            self.compression,
            Some(artifact_media_type),
            vm_target.clone(),
        )?;

        let policy = ContractPolicy::new(
            vec![vm_target.clone()],
            vec![artifact.artifact_format.clone()],
            self.max_artifact_size,
            self.allowed_capabilities,
            self.forbidden_capabilities,
            self.review_required,
            self.signature_required,
            self.source_trust_level.clone(),
        )?;

        let compatibility = Compatibility::new(
            self.minimum_schema_version,
            self.supported_schema_versions,
            supported_runtime_families,
            supported_network_classes,
            vec![],
            false,
        )?;

        let metadata = self
            .metadata
            .unwrap_or_else(|| default_metadata_for_name(&name));

        let integrity = Integrity {
            digest: digest.clone(),
            artifact_size: artifact.artifact_size,
            artifact_format: artifact.artifact_format.clone(),
            media_type: artifact.media_type.clone(),
            signature_required: self.signature_required,
            source_trust_level: self.source_trust_level,
        };

        let mut manifest = ContractManifest::new(
            name,
            package,
            version,
            contract_version,
            vm_target,
            artifact,
            self.entrypoints,
            digest,
            metadata,
            policy,
            compatibility,
            integrity,
            self.schema_version,
        )?;

        if let Some(execution_profile) = self.execution_profile {
            manifest.execution_profile = execution_profile;
        }
        if let Some(contract_class) = self.contract_class {
            manifest.execution_profile.contract_class = contract_class;
        }
        if let Some(capability_profile) = self.capability_profile {
            manifest.execution_profile.capability_profile = capability_profile;
        }
        if let Some(policy_profile) = self.policy_profile {
            manifest.execution_profile.policy_profile = policy_profile;
        }
        manifest.validate()?;

        Ok(manifest)
    }

    /// Builds a validated descriptor by wrapping the built manifest.
    pub fn build_descriptor(self) -> Result<ContractDescriptor, BuilderError> {
        let manifest = self.build()?;
        Ok(ContractDescriptor::new(manifest)?)
    }
}

/// Returns a required builder field or a deterministic `MissingField` error.
fn required<T>(value: Option<T>, field: &'static str) -> Result<T, BuilderError> {
    value.ok_or(BuilderError::MissingField(field))
}

fn normalize_supported_runtime_families(
    families: Vec<RuntimeFamily>,
    vm_target: &VmTarget,
) -> Vec<RuntimeFamily> {
    if families.is_empty() {
        vec![runtime_family_for_vm(vm_target)]
    } else {
        families
    }
}

fn normalize_supported_network_classes(classes: Vec<NetworkClass>) -> Vec<NetworkClass> {
    if classes.is_empty() {
        default_supported_network_classes()
    } else {
        classes
    }
}

fn default_media_type_for_format(format: &ArtifactFormat) -> &'static str {
    match format {
        ArtifactFormat::EvmBytecode => "application/octet-stream",
        ArtifactFormat::WasmModule => "application/wasm",
        ArtifactFormat::Archive => "application/vnd.aox.archive",
        ArtifactFormat::ManifestLinked => "application/json",
    }
}

fn default_supported_network_classes() -> Vec<NetworkClass> {
    vec![
        NetworkClass::Mainnet,
        NetworkClass::Testnet,
        NetworkClass::Devnet,
    ]
}

fn artifact_format_for_vm(vm_target: &VmTarget) -> ArtifactFormat {
    match vm_target {
        VmTarget::Evm => ArtifactFormat::EvmBytecode,
        _ => ArtifactFormat::WasmModule,
    }
}

fn runtime_family_for_vm(vm_target: &VmTarget) -> RuntimeFamily {
    match vm_target {
        VmTarget::Evm => RuntimeFamily::Evm,
        VmTarget::Wasm => RuntimeFamily::Wasm,
        VmTarget::SuiLike | VmTarget::Custom(_) => RuntimeFamily::AoxVm,
    }
}

fn default_metadata_for_name(name: &str) -> ContractMetadata {
    ContractMetadata {
        display_name: name.to_string(),
        description: Some("Generated by AOXC SDK builder".to_string()),
        author: None,
        organization: None,
        source_reference: None,
        tags: vec!["sdk".to_string()],
        created_at: None,
        updated_at: None,
        audit_reference: None,
        notes: None,
    }
}

