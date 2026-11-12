// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

//! Persistent node identity and explicit peer pinning.
//!
//! A node owns one long-term Ed25519 identity, stored in a private key file that
//! is created only by an explicit provisioning call and never overwritten. Remote
//! peers are authorised solely by their public key appearing in a [`TrustedPeers`]
//! set that the operator filled from keys exchanged out of band. An empty or
//! missing set authorises nobody; there is no trust on first use.

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand_core::{OsRng, RngCore};
use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use zeroize::{Zeroize, Zeroizing};

/// First line of an identity key file.
const KEY_FILE_HEADER: &str = "cryprq-ed25519-identity-v1";
/// Prefix of a textual public key.
const PUBLIC_KEY_PREFIX: &str = "ed25519:";
/// An identity key file is exactly header, newline, 64 hex digits, newline.
const KEY_FILE_LEN: usize = KEY_FILE_HEADER.len() + 1 + 64 + 1;
/// Upper bound on the size of a trusted-peers file.
const MAX_TRUST_FILE_BYTES: u64 = 256 * 1024;
/// Upper bound on the number of pinned peers.
pub const MAX_TRUSTED_PEERS: usize = 1024;

/// Errors from identity storage and trust configuration.
///
/// Messages never contain key material or file contents.
#[derive(Debug)]
pub enum IdentityError {
    /// The key or trust file does not exist.
    NotFound(String),
    /// Refusing to replace an identity that already exists.
    AlreadyExists(String),
    /// The file or its directory is accessible to other users, or is not a
    /// regular file.
    UnsafeStorage(String),
    /// The identity key file is not in the expected format.
    MalformedKeyFile,
    /// A public key is not a valid, non-weak Ed25519 key.
    InvalidPublicKey,
    /// The trusted-peers file is invalid; the 1-based line number is given.
    MalformedTrustFile(usize),
    /// The trusted-peers file pins the same key twice.
    DuplicatePeer(usize),
    /// The trusted-peers file exceeds a size or entry limit.
    TooLarge,
    /// Underlying I/O failure.
    Io(std::io::ErrorKind),
}

impl fmt::Display for IdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "{path}: not found"),
            Self::AlreadyExists(path) => {
                write!(f, "{path}: identity already exists; refusing to overwrite")
            }
            Self::UnsafeStorage(why) => write!(f, "unsafe key storage: {why}"),
            Self::MalformedKeyFile => write!(f, "identity key file is malformed"),
            Self::InvalidPublicKey => write!(f, "invalid Ed25519 public key"),
            Self::MalformedTrustFile(line) => {
                write!(f, "trusted-peers file is malformed at line {line}")
            }
            Self::DuplicatePeer(line) => {
                write!(f, "trusted-peers file repeats a key at line {line}")
            }
            Self::TooLarge => write!(f, "trusted-peers file exceeds the supported size"),
            Self::Io(kind) => write!(f, "I/O error: {kind}"),
        }
    }
}

impl std::error::Error for IdentityError {}

fn io_error(path: &Path, err: std::io::Error) -> IdentityError {
    match err.kind() {
        std::io::ErrorKind::NotFound => IdentityError::NotFound(path.display().to_string()),
        std::io::ErrorKind::AlreadyExists => {
            IdentityError::AlreadyExists(path.display().to_string())
        }
        kind => IdentityError::Io(kind),
    }
}

/// A validated Ed25519 public identity key of a peer.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PeerPublicKey([u8; 32]);

impl PeerPublicKey {
    /// Accepts only canonical, non-weak Ed25519 public keys.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, IdentityError> {
        let key = VerifyingKey::from_bytes(bytes).map_err(|_| IdentityError::InvalidPublicKey)?;
        if key.is_weak() {
            return Err(IdentityError::InvalidPublicKey);
        }
        Ok(Self(*bytes))
    }

    /// Parses the textual form `ed25519:<64 lowercase or uppercase hex digits>`.
    pub fn parse(text: &str) -> Result<Self, IdentityError> {
        let hex_part = text
            .strip_prefix(PUBLIC_KEY_PREFIX)
            .ok_or(IdentityError::InvalidPublicKey)?;
        if hex_part.len() != 64 {
            return Err(IdentityError::InvalidPublicKey);
        }
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(hex_part, &mut bytes).map_err(|_| IdentityError::InvalidPublicKey)?;
        Self::from_bytes(&bytes)
    }

    /// Raw key bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Strictly verifies `signature` over `message` under this key.
    pub fn verify(&self, message: &[u8], signature: &[u8; 64]) -> bool {
        match VerifyingKey::from_bytes(&self.0) {
            Ok(key) => key
                .verify_strict(message, &Signature::from_bytes(signature))
                .is_ok(),
            Err(_) => false,
        }
    }
}

impl fmt::Display for PeerPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{PUBLIC_KEY_PREFIX}{}", hex::encode(self.0))
    }
}

impl fmt::Debug for PeerPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// The node's long-term Ed25519 identity.
///
/// The secret key is zeroized on drop and is never printed.
pub struct NodeIdentity {
    signing: SigningKey,
}

impl fmt::Debug for NodeIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeIdentity({})", self.public_key())
    }
}

impl NodeIdentity {
    /// Provisions a new identity and stores it at `path`.
    ///
    /// This is the only way an identity file comes into existence. It fails if
    /// anything already exists at `path`. Missing parent directories are created
    /// accessible to the owner only, and the key file is created with mode 0600.
    pub fn generate_new(path: &Path) -> Result<Self, IdentityError> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            if !parent.exists() {
                create_private_dir(parent)?;
            }
            check_dir_is_private(parent)?;
        }

        let mut seed = Zeroizing::new([0u8; 32]);
        OsRng.fill_bytes(seed.as_mut());
        let identity = Self::from_secret_seed(&seed);

        let mut contents = Zeroizing::new(String::with_capacity(KEY_FILE_LEN));
        contents.push_str(KEY_FILE_HEADER);
        contents.push('\n');
        contents.push_str(&Zeroizing::new(hex::encode(seed.as_ref())));
        contents.push('\n');

        let mut file = create_private_file(path)?;
        file.write_all(contents.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|e| io_error(path, e))?;
        Ok(identity)
    }

    /// Loads the identity stored at `path`.
    ///
    /// Fails closed: a missing, malformed, symlinked, or (on Unix) group- or
    /// world-accessible key file is rejected, and no identity is generated.
    pub fn load(path: &Path) -> Result<Self, IdentityError> {
        let metadata = fs::symlink_metadata(path).map_err(|e| io_error(path, e))?;
        if !metadata.file_type().is_file() {
            return Err(IdentityError::UnsafeStorage(format!(
                "{} is not a regular file",
                path.display()
            )));
        }
        check_file_is_private(path, &metadata)?;
        if metadata.len() != KEY_FILE_LEN as u64 {
            return Err(IdentityError::MalformedKeyFile);
        }

        let mut contents = Zeroizing::new(Vec::with_capacity(KEY_FILE_LEN + 1));
        fs::File::open(path)
            .and_then(|f| f.take(KEY_FILE_LEN as u64 + 1).read_to_end(&mut contents))
            .map_err(|e| io_error(path, e))?;
        let seed = parse_key_file(&contents)?;
        Ok(Self::from_secret_seed(&seed))
    }

    /// Builds an identity from a 32-byte Ed25519 seed.
    ///
    /// For hosts that keep the seed in their own secure storage (for example a
    /// platform keychain). The caller remains responsible for the seed's secrecy.
    pub fn from_secret_seed(seed: &[u8; 32]) -> Self {
        Self {
            signing: SigningKey::from_bytes(seed),
        }
    }

    /// The public half of this identity, safe to share.
    pub fn public_key(&self) -> PeerPublicKey {
        PeerPublicKey(self.signing.verifying_key().to_bytes())
    }

    /// Signs `message`. Callers must prefix a domain-separation context.
    pub(crate) fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.signing.sign(message).to_bytes()
    }

    /// The Ed25519 seed, for binding a transport identity (such as a libp2p
    /// keypair) to this same key. Handle with the care due to a private key.
    pub fn secret_seed(&self) -> Zeroizing<[u8; 32]> {
        Zeroizing::new(self.signing.to_bytes())
    }
}

fn parse_key_file(contents: &[u8]) -> Result<Zeroizing<[u8; 32]>, IdentityError> {
    if contents.len() != KEY_FILE_LEN {
        return Err(IdentityError::MalformedKeyFile);
    }
    let (header, rest) = contents.split_at(KEY_FILE_HEADER.len());
    if header != KEY_FILE_HEADER.as_bytes() || rest[0] != b'\n' || rest[65] != b'\n' {
        return Err(IdentityError::MalformedKeyFile);
    }
    let mut seed = Zeroizing::new([0u8; 32]);
    if hex::decode_to_slice(&rest[1..65], seed.as_mut()).is_err() {
        seed.zeroize();
        return Err(IdentityError::MalformedKeyFile);
    }
    Ok(seed)
}

#[cfg(unix)]
fn create_private_dir(dir: &Path) -> Result<(), IdentityError> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)
        .map_err(|e| io_error(dir, e))
}

#[cfg(not(unix))]
fn create_private_dir(dir: &Path) -> Result<(), IdentityError> {
    fs::create_dir_all(dir).map_err(|e| io_error(dir, e))
}

#[cfg(unix)]
fn create_private_file(path: &Path) -> Result<fs::File, IdentityError> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|e| io_error(path, e))
}

#[cfg(not(unix))]
fn create_private_file(path: &Path) -> Result<fs::File, IdentityError> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| io_error(path, e))
}

#[cfg(unix)]
fn check_file_is_private(path: &Path, metadata: &fs::Metadata) -> Result<(), IdentityError> {
    use std::os::unix::fs::PermissionsExt;
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err(IdentityError::UnsafeStorage(format!(
            "{} is accessible to group or others; expected mode 0600",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_file_is_private(_path: &Path, _metadata: &fs::Metadata) -> Result<(), IdentityError> {
    Ok(())
}

#[cfg(unix)]
fn check_dir_is_private(dir: &Path) -> Result<(), IdentityError> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = fs::metadata(dir).map_err(|e| io_error(dir, e))?;
    if !metadata.is_dir() {
        return Err(IdentityError::UnsafeStorage(format!(
            "{} is not a directory",
            dir.display()
        )));
    }
    if metadata.permissions().mode() & 0o022 != 0 {
        return Err(IdentityError::UnsafeStorage(format!(
            "directory {} is writable by group or others",
            dir.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_dir_is_private(_dir: &Path) -> Result<(), IdentityError> {
    Ok(())
}

/// The explicit set of peer identities this node will talk to.
///
/// The default set is empty and therefore rejects every peer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TrustedPeers {
    keys: BTreeSet<PeerPublicKey>,
}

impl TrustedPeers {
    /// An empty set, which trusts nobody.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pins the given keys.
    pub fn from_keys<I: IntoIterator<Item = PeerPublicKey>>(keys: I) -> Self {
        Self {
            keys: keys.into_iter().collect(),
        }
    }

    /// Loads pinned keys from a file with one `ed25519:<hex>` entry per line,
    /// optionally followed by whitespace and a free-form label. Blank lines and
    /// lines starting with `#` are ignored.
    ///
    /// Fails closed: a missing file or any invalid line rejects the whole file.
    /// On Unix the file must not be writable by group or others, because
    /// whoever can edit it decides who is trusted.
    pub fn load(path: &Path) -> Result<Self, IdentityError> {
        let metadata = fs::symlink_metadata(path).map_err(|e| io_error(path, e))?;
        if !metadata.file_type().is_file() {
            return Err(IdentityError::UnsafeStorage(format!(
                "{} is not a regular file",
                path.display()
            )));
        }
        check_file_not_writable_by_others(path, &metadata)?;
        if metadata.len() > MAX_TRUST_FILE_BYTES {
            return Err(IdentityError::TooLarge);
        }
        let mut text = String::new();
        fs::File::open(path)
            .and_then(|f| f.take(MAX_TRUST_FILE_BYTES + 1).read_to_string(&mut text))
            .map_err(|e| io_error(path, e))?;
        if text.len() as u64 > MAX_TRUST_FILE_BYTES {
            return Err(IdentityError::TooLarge);
        }
        Self::parse(&text)
    }

    /// Parses the trusted-peers file format; see [`TrustedPeers::load`].
    pub fn parse(text: &str) -> Result<Self, IdentityError> {
        let mut keys = BTreeSet::new();
        for (index, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let token = line.split_whitespace().next().unwrap_or_default();
            let key = PeerPublicKey::parse(token)
                .map_err(|_| IdentityError::MalformedTrustFile(index + 1))?;
            if !keys.insert(key) {
                return Err(IdentityError::DuplicatePeer(index + 1));
            }
            if keys.len() > MAX_TRUSTED_PEERS {
                return Err(IdentityError::TooLarge);
            }
        }
        Ok(Self { keys })
    }

    /// Whether `key` is explicitly pinned.
    pub fn contains(&self, key: &PeerPublicKey) -> bool {
        self.keys.contains(key)
    }

    /// Number of pinned peers.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// True when nobody is trusted.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Iterates over the pinned keys.
    pub fn iter(&self) -> impl Iterator<Item = &PeerPublicKey> {
        self.keys.iter()
    }
}

#[cfg(unix)]
fn check_file_not_writable_by_others(
    path: &Path,
    metadata: &fs::Metadata,
) -> Result<(), IdentityError> {
    use std::os::unix::fs::PermissionsExt;
    if metadata.permissions().mode() & 0o022 != 0 {
        return Err(IdentityError::UnsafeStorage(format!(
            "{} is writable by group or others",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_file_not_writable_by_others(
    _path: &Path,
    _metadata: &fs::Metadata,
) -> Result<(), IdentityError> {
    Ok(())
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use std::path::PathBuf;

    /// A private scratch directory removed on drop.
    pub(crate) struct ScratchDir(pub PathBuf);

    impl ScratchDir {
        pub(crate) fn new(tag: &str) -> Self {
            let mut suffix = [0u8; 8];
            OsRng.fill_bytes(&mut suffix);
            let path = std::env::temp_dir().join(format!(
                "cryprq-test-{tag}-{}-{}",
                std::process::id(),
                hex::encode(suffix)
            ));
            create_private_dir(&path).expect("create scratch dir");
            Self(path)
        }

        pub(crate) fn path(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for ScratchDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// An in-memory identity for tests that do not exercise storage.
    pub(crate) fn ephemeral_identity() -> NodeIdentity {
        let mut seed = Zeroizing::new([0u8; 32]);
        OsRng.fill_bytes(seed.as_mut());
        NodeIdentity::from_secret_seed(&seed)
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::test_support::{ephemeral_identity, ScratchDir};
    use super::*;

    #[test]
    fn identity_persists_across_reload() {
        let dir = ScratchDir::new("persist");
        let path = dir.path("identity.key");
        let created = NodeIdentity::generate_new(&path).expect("provision");
        let first = NodeIdentity::load(&path).expect("first load");
        let second = NodeIdentity::load(&path).expect("second load");
        assert_eq!(created.public_key(), first.public_key());
        assert_eq!(first.public_key(), second.public_key());
        // Same key, not merely the same label: signatures verify across reloads.
        let signature = first.sign(b"restart");
        assert!(second.public_key().verify(b"restart", &signature));
    }

    #[test]
    fn missing_identity_fails_closed_without_generating() {
        let dir = ScratchDir::new("missing");
        let path = dir.path("absent.key");
        assert!(matches!(
            NodeIdentity::load(&path),
            Err(IdentityError::NotFound(_))
        ));
        assert!(!path.exists(), "load must never create an identity");
    }

    #[test]
    fn existing_identity_is_never_overwritten() {
        let dir = ScratchDir::new("overwrite");
        let path = dir.path("identity.key");
        let original = NodeIdentity::generate_new(&path).expect("provision");
        let before = fs::read(&path).expect("read");
        assert!(matches!(
            NodeIdentity::generate_new(&path),
            Err(IdentityError::AlreadyExists(_))
        ));
        assert_eq!(before, fs::read(&path).expect("read"));
        assert_eq!(
            original.public_key(),
            NodeIdentity::load(&path).expect("load").public_key()
        );
    }

    #[test]
    fn independently_provisioned_identities_differ() {
        let dir = ScratchDir::new("distinct");
        let a = NodeIdentity::generate_new(&dir.path("a.key")).expect("a");
        let b = NodeIdentity::generate_new(&dir.path("b.key")).expect("b");
        assert_ne!(a.public_key(), b.public_key());
    }

    #[test]
    fn malformed_identity_files_are_rejected() {
        let dir = ScratchDir::new("malformed");
        let good_path = dir.path("good.key");
        NodeIdentity::generate_new(&good_path).expect("provision");
        let good = fs::read(&good_path).expect("read");

        let mut wrong_header = good.clone();
        wrong_header[0] ^= 0x20;
        let mut bad_hex = good.clone();
        bad_hex[KEY_FILE_HEADER.len() + 1] = b'z';
        let mut no_newline = good.clone();
        *no_newline.last_mut().expect("nonempty") = b' ';
        let cases: Vec<(&str, Vec<u8>)> = vec![
            ("empty", Vec::new()),
            ("truncated", good[..good.len() - 2].to_vec()),
            ("extended", [good.clone(), b"\n".to_vec()].concat()),
            ("wrong-header", wrong_header),
            ("bad-hex", bad_hex),
            ("no-newline", no_newline),
            ("raw-seed", vec![7u8; 32]),
        ];
        for (name, bytes) in cases {
            let path = dir.path(name);
            create_private_file(&path)
                .and_then(|mut f| f.write_all(&bytes).map_err(|e| io_error(&path, e)))
                .expect("write case");
            assert!(
                matches!(
                    NodeIdentity::load(&path),
                    Err(IdentityError::MalformedKeyFile)
                ),
                "case {name} must be rejected as malformed"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn unsafe_identity_storage_is_rejected() {
        use std::os::unix::fs::PermissionsExt;
        let dir = ScratchDir::new("perms");
        let path = dir.path("identity.key");
        NodeIdentity::generate_new(&path).expect("provision");
        assert_eq!(
            fs::metadata(&path).expect("stat").permissions().mode() & 0o777,
            0o600
        );

        for mode in [0o640, 0o604, 0o660, 0o644] {
            fs::set_permissions(&path, fs::Permissions::from_mode(mode)).expect("chmod");
            assert!(
                matches!(
                    NodeIdentity::load(&path),
                    Err(IdentityError::UnsafeStorage(_))
                ),
                "mode {mode:o} must be rejected"
            );
        }
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("chmod");

        let link = dir.path("link.key");
        std::os::unix::fs::symlink(&path, &link).expect("symlink");
        assert!(matches!(
            NodeIdentity::load(&link),
            Err(IdentityError::UnsafeStorage(_))
        ));
        assert!(matches!(
            NodeIdentity::load(&dir.0),
            Err(IdentityError::UnsafeStorage(_))
        ));

        // Provisioning refuses a directory other users can write into.
        let shared = dir.path("shared");
        fs::create_dir(&shared).expect("mkdir");
        fs::set_permissions(&shared, fs::Permissions::from_mode(0o777)).expect("chmod");
        let target = shared.join("identity.key");
        assert!(matches!(
            NodeIdentity::generate_new(&target),
            Err(IdentityError::UnsafeStorage(_))
        ));
        assert!(!target.exists());
    }

    #[cfg(unix)]
    #[test]
    fn provisioning_creates_private_parent_directories() {
        use std::os::unix::fs::PermissionsExt;
        let dir = ScratchDir::new("parents");
        let path = dir.path("nested/deeper/identity.key");
        NodeIdentity::generate_new(&path).expect("provision");
        let parent = path.parent().expect("parent");
        assert_eq!(
            fs::metadata(parent).expect("stat").permissions().mode() & 0o777,
            0o700
        );
        NodeIdentity::load(&path).expect("load");
    }

    #[test]
    fn errors_and_debug_output_do_not_reveal_the_seed() {
        let dir = ScratchDir::new("leak");
        let path = dir.path("identity.key");
        let identity = NodeIdentity::generate_new(&path).expect("provision");
        let seed_hex = hex::encode(identity.secret_seed().as_ref());
        assert!(!format!("{identity:?}").contains(&seed_hex));

        // Corrupt one hex digit and confirm the error does not echo the contents.
        let mut bytes = fs::read(&path).expect("read");
        bytes[KEY_FILE_HEADER.len() + 5] = b'!';
        fs::remove_file(&path).expect("remove");
        create_private_file(&path)
            .and_then(|mut f| f.write_all(&bytes).map_err(|e| io_error(&path, e)))
            .expect("rewrite");
        let err = NodeIdentity::load(&path).expect_err("malformed");
        let rendered = format!("{err} {err:?}");
        assert!(!rendered.contains(&seed_hex[8..40]));
    }

    #[test]
    fn empty_trust_set_rejects_everyone() {
        let peer = ephemeral_identity().public_key();
        assert!(TrustedPeers::default().is_empty());
        assert!(!TrustedPeers::default().contains(&peer));
        assert!(!TrustedPeers::new().contains(&peer));
        assert!(!TrustedPeers::parse("").expect("empty").contains(&peer));
        assert!(!TrustedPeers::parse("# only a comment\n\n")
            .expect("comment")
            .contains(&peer));
    }

    #[test]
    fn only_explicitly_pinned_peers_are_trusted() {
        let pinned = ephemeral_identity().public_key();
        let other = ephemeral_identity().public_key();
        let trust =
            TrustedPeers::parse(&format!("# laptop\n{pinned} laptop of alice\n")).expect("parse");
        assert_eq!(trust.len(), 1);
        assert!(trust.contains(&pinned));
        assert!(!trust.contains(&other));

        // A substituted key that differs in a single bit is not the pinned key.
        let mut substituted = *pinned.as_bytes();
        substituted[31] ^= 0x01;
        if let Ok(key) = PeerPublicKey::from_bytes(&substituted) {
            assert!(!trust.contains(&key));
        }
    }

    #[test]
    fn malformed_trust_configuration_is_rejected_entirely() {
        let good = ephemeral_identity().public_key();
        let hex_only = hex::encode(good.as_bytes());
        let cases = [
            hex_only.clone(),                         // missing prefix
            format!("ed25519:{}", &hex_only[..62]),   // too short
            format!("ed25519:{hex_only}00"),          // too long
            format!("ed25519:{}zz", &hex_only[..62]), // not hex
            format!("rsa:{hex_only}"),                // wrong algorithm
            format!("ed25519:{}", "00".repeat(32)),   // weak (identity point)
            "*".to_string(),                          // no wildcards
            "any".to_string(),
        ];
        for case in cases {
            let text = format!("{good}\n{case}\n");
            assert!(
                matches!(
                    TrustedPeers::parse(&text),
                    Err(IdentityError::MalformedTrustFile(2))
                ),
                "entry {case:?} must invalidate the file"
            );
        }
        assert!(matches!(
            TrustedPeers::parse(&format!("{good}\n{good}\n")),
            Err(IdentityError::DuplicatePeer(2))
        ));
    }

    #[test]
    fn missing_trust_file_fails_closed() {
        let dir = ScratchDir::new("trust-missing");
        assert!(matches!(
            TrustedPeers::load(&dir.path("peers.txt")),
            Err(IdentityError::NotFound(_))
        ));
    }

    #[test]
    fn trust_file_round_trips_and_enforces_limits() {
        let dir = ScratchDir::new("trust-file");
        let a = ephemeral_identity().public_key();
        let b = ephemeral_identity().public_key();
        let path = dir.path("peers.txt");
        create_private_file(&path)
            .and_then(|mut f| {
                f.write_all(format!("{a} a\n\n# b follows\n  {b}\n").as_bytes())
                    .map_err(|e| io_error(&path, e))
            })
            .expect("write");
        let trust = TrustedPeers::load(&path).expect("load");
        assert_eq!(trust, TrustedPeers::from_keys([a, b]));

        let big = dir.path("big.txt");
        create_private_file(&big)
            .and_then(|mut f| {
                f.write_all(&vec![b'#'; MAX_TRUST_FILE_BYTES as usize + 1])
                    .map_err(|e| io_error(&big, e))
            })
            .expect("write");
        assert!(matches!(
            TrustedPeers::load(&big),
            Err(IdentityError::TooLarge)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn trust_file_writable_by_others_is_rejected() {
        use std::os::unix::fs::PermissionsExt;
        let dir = ScratchDir::new("trust-perms");
        let path = dir.path("peers.txt");
        let key = ephemeral_identity().public_key();
        create_private_file(&path)
            .and_then(|mut f| {
                f.write_all(format!("{key}\n").as_bytes())
                    .map_err(|e| io_error(&path, e))
            })
            .expect("write");
        // Public material may be world-readable, but not world-writable.
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("chmod");
        assert!(TrustedPeers::load(&path).is_ok());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).expect("chmod");
        assert!(matches!(
            TrustedPeers::load(&path),
            Err(IdentityError::UnsafeStorage(_))
        ));
    }

    #[test]
    fn public_key_text_round_trips_and_rejects_invalid_points() {
        let key = ephemeral_identity().public_key();
        assert_eq!(PeerPublicKey::parse(&key.to_string()).expect("parse"), key);
        assert!(PeerPublicKey::from_bytes(&[0u8; 32]).is_err());
        let mut identity_point = [0u8; 32];
        identity_point[0] = 1;
        assert!(PeerPublicKey::from_bytes(&identity_point).is_err());
    }

    #[test]
    fn signatures_verify_only_under_the_signing_identity() {
        let signer = ephemeral_identity();
        let imposter = ephemeral_identity();
        let signature = signer.sign(b"message");
        assert!(signer.public_key().verify(b"message", &signature));
        assert!(!imposter.public_key().verify(b"message", &signature));
        assert!(!signer.public_key().verify(b"other", &signature));
        let mut tampered = signature;
        tampered[10] ^= 0x40;
        assert!(!signer.public_key().verify(b"message", &tampered));
    }

    /// Scans every Git-tracked file for something shaped like an identity key
    /// file, so a private key cannot be committed unnoticed.
    #[test]
    fn no_identity_private_key_is_tracked_in_git() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let listing = std::process::Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["ls-files", "-z"])
            .output()
            .expect("git must be available to audit tracked files");
        assert!(listing.status.success(), "git ls-files failed");
        let needle = format!("{KEY_FILE_HEADER}\n");
        let mut scanned = 0usize;
        for name in listing.stdout.split(|b| *b == 0).filter(|n| !n.is_empty()) {
            let path = root.join(String::from_utf8_lossy(name).as_ref());
            let Ok(bytes) = fs::read(&path) else { continue };
            scanned += 1;
            let mut offset = 0;
            while let Some(found) = find(&bytes[offset..], needle.as_bytes()) {
                let start = offset + found + needle.len();
                let candidate = bytes.get(start..start + 64).unwrap_or_default();
                assert!(
                    !(candidate.len() == 64 && candidate.iter().all(u8::is_ascii_hexdigit)),
                    "tracked file {} contains an identity private key",
                    path.display()
                );
                offset = start;
            }
        }
        assert!(
            scanned > 50,
            "expected to scan the repository, saw {scanned}"
        );
    }

    fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack.windows(needle.len()).position(|w| w == needle)
    }
}
