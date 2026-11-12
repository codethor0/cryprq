# CrypRQ v1.0 Protocol Specification

**Document Version:** 1.0.1

**Date:** November 14, 2025

**Authors:** Senior Cryptographic Protocol Designer

**Abstract:** This document specifies the CrypRQ v1.0 protocol, a post-quantum-hybrid secure tunnel and file-transfer protocol designed for peer-to-peer communication. It leverages a hybrid key exchange combining ML-KEM (Kyber768) and X25519 to establish secure, ephemeral session keys, ensuring confidentiality and integrity of data in transit. The protocol is designed to operate primarily over QUIC/UDP but is transport-agnostic, supporting future extensions to TCP and WebSocket. This specification details the handshake process, key schedule, record layer, message types, and security considerations to enable interoperable implementations.

## 1. Introduction and Overview

### 1.1. Purpose and Scope

This document provides a comprehensive, implementation-independent specification for the CrypRQ v1.0 protocol. Its primary purpose is to serve as the canonical reference for developers building compatible applications and services. The protocol is engineered to provide robust, forward-secure, and post-quantum-resistant encrypted communication channels for a variety of use cases, including secure file transfer, generic encrypted tunnels for arbitrary data streams, and Virtual Private Network (VPN) functionality. The scope of this document covers the cryptographic handshake, the key derivation and rotation mechanisms, the structure of data frames (records), the semantics of different message types, and the overall security model. It is intended to be a precise and unambiguous guide, enabling independent teams to create interoperable implementations without needing to consult the original source code. The specification is written in the style of an IETF RFC, utilizing key words such as **MUST**, **MUST NOT**, **SHOULD**, and **MAY** as defined in RFC 2119 and RFC 8174 to indicate requirement levels.

### 1.2. Protocol Goals

The fundamental goal of CrypRQ v1.0 is to establish a secure, high-performance, and resilient communication channel between two peers. The protocol aims to achieve several specific objectives. First and foremost is **confidentiality**: ensuring that all data transmitted between peers is encrypted and unreadable to any unauthorized third party. This is achieved through the use of strong, authenticated encryption with associated data (AEAD) ciphers. Second is **integrity**: guaranteeing that data is not tampered with or altered in transit without detection. The AEAD construction provides this property. Third is **authenticity**: verifying the identity of the communicating peers, which is established during the cryptographic handshake. A critical and distinguishing goal is **post-quantum security**, which is addressed by incorporating a hybrid key exchange mechanism that combines the classical X25519 elliptic-curve Diffie-Hellman algorithm with the post-quantum ML-KEM (Kyber768) key encapsulation mechanism. This hybrid approach ensures that the session remains secure even if one of the underlying cryptographic primitives is compromised in the future. Finally, the protocol aims for **forward secrecy** and **ephemeral key rotation**, where session keys are periodically refreshed to limit the impact of a potential long-term key compromise and to mitigate the risk of mass surveillance.

### 1.3. Protocol Non-Goals

While CrypRQ v1.0 is designed to be a robust security protocol, it is crucial to understand its limitations and the features that are explicitly outside its scope for this version. **Anonymity** is a non-goal; the protocol does not attempt to hide the identities of the communicating peers or their IP addresses from a network observer. It focuses on securing the content of the communication, not the metadata. Similarly, **traffic-analysis resistance** is not provided. The protocol does not employ mechanisms to obscure the size, timing, or frequency of packets, which could potentially be used by an adversary to infer the nature of the communication (e.g., distinguishing between file transfer and web browsing). **Censorship resistance** is also not a design objective. While the encrypted content is unreadable, the protocol's traffic patterns and handshake signatures may be identifiable by deep packet inspection systems, making it susceptible to blocking by sophisticated censors. Furthermore, CrypRQ v1.0 does not include features like built-in support for TLS wrapping, DNS-over-TLS integration, or formal Known Answer Test (KAT) vectors for cryptographic primitives, which are considered future enhancements.

### 1.4. High-Level Overview

CrypRQ v1.0 operates as a layer above a transport protocol, primarily QUIC/UDP, to provide a secure channel for application data. The protocol's lifecycle begins with a **hybrid cryptographic handshake**. During this phase, two peers, an Initiator and a Responder, perform a key exchange using both the post-quantum ML-KEM (Kyber768) and the classical X25519 algorithms. This dual approach ensures a high level of security, resistant to both current and future cryptanalytic attacks, including those from quantum computers. The shared secrets derived from both key exchanges are combined using a Key Derivation Function (KDF) to generate a master secret. From this master secret, a set of **ephemeral application traffic keys** are derived for encrypting and authenticating subsequent data. A key feature of the protocol is its **automatic key rotation** mechanism. These application keys are periodically refreshed (by default, every five minutes) without requiring a new handshake, thereby enhancing forward secrecy. Once the handshake is complete and keys are established, application data is encapsulated in **CrypRQ records**. Each record contains a header with metadata (such as message type, stream ID, and sequence number) and a ciphertext payload, which is the encrypted application data. The protocol supports multiple concurrent data streams, enabling different types of traffic (e.g., file transfer, VPN packets, control messages) to be multiplexed over a single secure session.

## 2. Terminology and Roles

### 2.1. Key Terms

This section defines the key terminology used throughout this specification to ensure clarity and consistency.

*   **Peer:** An endpoint participating in a CrypRQ communication session. Each peer is considered equal in capability and can act as both a sender and a receiver of data.

*   **Session:** A long-lived, secure communication context established between two peers. A session is initiated by the handshake protocol and is maintained until it is explicitly closed or terminated due to an error. All communication, including key rotation and data transfer, occurs within the context of a session.

*   **Stream:** A logical, bidirectional or unidirectional channel for data flow within a CrypRQ session. Multiple streams can be active simultaneously over a single session, allowing for the multiplexing of different types of data (e.g., one stream for a file transfer, another for VPN traffic). Each stream is identified by a unique Stream ID.

*   **Tunnel:** A generic term for the encrypted communication channel provided by a CrypRQ session. It represents the secure pathway through which arbitrary byte streams are transmitted between peers.

*   **File Transfer Session:** A specific type of stream used for transferring files. It involves a defined sequence of messages, including metadata exchange (`FILE_META`), chunked data transfer (`FILE_CHUNK`), acknowledgments (`FILE_ACK`), and final integrity verification using a SHA-256 hash.

*   **VPN Mode:** An operational mode where CrypRQ is used to create a Virtual Private Network. In this mode, the protocol interfaces with a TUN (network tunnel) interface to capture and transmit raw IP packets, effectively creating a secure, system-wide network link between peers.

*   **Epoch:** A numerical counter that identifies a specific generation of traffic keys within a session. The epoch number is incremented with each key rotation event. It is included in the record header to allow the receiver to select the correct key for decryption.

*   **Handshake Keys:** Cryptographic keys derived during the initial handshake process. These keys are typically used to encrypt and authenticate the final handshake messages themselves, providing an additional layer of security before the application traffic keys are established.

*   **Application Keys:** The primary symmetric keys used for encrypting and authenticating all application data (e.g., `DATA`, `FILE_CHUNK`, `VPN_PACKET` messages) exchanged during a session. These keys are derived from the handshake's master secret and are periodically rotated.

### 2.2. Peer Roles and Symmetry

The CrypRQ protocol is fundamentally symmetric, meaning that both peers in a connection have identical capabilities and responsibilities. There is no inherent client or server role. Either peer can initiate a connection, request a file transfer, or establish a VPN tunnel. This design simplifies the protocol and promotes a decentralized, peer-to-peer architecture. However, for the purpose of describing the handshake process and the flow of messages, it is often convenient to assign temporary, descriptive roles. The peer that initiates the connection and sends the first handshake message is referred to as the **Initiator**. The peer that receives this initial message and responds is known as the **Responder**. These roles are only relevant during the handshake phase and do not imply any long-term hierarchy or difference in functionality. Once the session is established, both peers operate under the same set of rules, using their respective application keys to secure their transmissions.

### 2.3. Session and Stream Concepts

A **CrypRQ session** represents the top-level secure association between two peers. It is established through the initial handshake and persists until it is terminated. The session encompasses the shared cryptographic state, including the current set of traffic keys and the epoch counter. All communication between the peers is bound to a specific session. Within a session, data is exchanged over **streams**. A stream provides a lightweight, independent channel for communication. This multiplexing capability is a key feature of the protocol, as it allows different types of traffic to be handled concurrently without head-of-line blocking between streams. For example, a large file transfer on one stream will not prevent small, time-sensitive control messages from being delivered on another. Each stream is assigned a unique 32-bit identifier, the Stream ID, which is included in the record header. This allows the receiver to demultiplex incoming records and route them to the correct application-level handler. Streams can be created implicitly by sending a message with a new Stream ID or explicitly through a control message. They can also be closed gracefully, ensuring that all data is delivered, or forcibly, in case of an error.

## 3. Protocol Stack and Transports

### 3.1. Transport Layer Requirements

CrypRQ is designed to be largely independent of the underlying transport protocol, but it does make certain assumptions about the capabilities of the transport layer. The primary requirement is that the transport **MUST** provide a reliable, ordered, and error-checked delivery mechanism for data. This means that the transport layer is responsible for handling packet loss, reordering, and retransmission. The CrypRQ record layer does not implement its own mechanisms for these issues, relying instead on the guarantees provided by the transport. This design choice simplifies the protocol and allows it to leverage the performance and efficiency of modern transport protocols like QUIC. Additionally, the transport layer **MUST** provide a datagram or byte-stream abstraction that can carry the binary CrypRQ records. The transport **SHOULD** also provide congestion control to ensure fair sharing of network resources. While the protocol is designed with these requirements in mind, the specific details of how the transport meets them are outside the scope of this specification.

### 3.2. QUIC/UDP as the Primary Transport

The primary and recommended transport for CrypRQ v1.0 is **QUIC over UDP**. QUIC is a modern, secure, and multiplexed transport protocol that inherently provides the reliability, ordering, and congestion control required by the CrypRQ record layer. By building on top of QUIC, CrypRQ can focus on its core competencies: key management, encryption, and application-level framing. QUIC's ability to handle multiple streams within a single connection aligns perfectly with CrypRQ's stream multiplexing feature, allowing for efficient and low-latency communication. The use of UDP as the underlying network protocol allows for greater flexibility and can help avoid some of the issues associated with TCP, such as head-of-line blocking at the transport layer. When running over QUIC, each CrypRQ record is typically encapsulated within a single QUIC STREAM frame. The combination of QUIC's transport security features and CrypRQ's application-layer cryptography provides a robust defense-in-depth security model.

### 3.3. Future Transport Considerations

While QUIC/UDP is the primary transport for v1.0, the protocol is designed to be extensible to other transports in the future. The specification explicitly mentions **TCP** and **WebSocket** as potential future transports. To run over TCP, the CrypRQ record layer would need to be modified to handle its own framing, as TCP is a byte-stream protocol and does not preserve message boundaries. This would likely involve adding a length field to the record header to indicate the size of each record. Running over WebSocket would be similar, with CrypRQ records being encapsulated within WebSocket frames. The core cryptographic and key management aspects of the protocol would remain unchanged, regardless of the transport. The choice of transport would primarily affect the lower-level framing and the mechanisms for handling reliability and congestion. The protocol's design ensures that the application-level logic and security properties are decoupled from the specifics of the transport layer.

### 3.4. MTU and Fragmentation

The Maximum Transmission Unit (MTU) of the network path is a critical consideration for any protocol. CrypRQ records **SHOULD** be sized to fit within the path MTU to avoid IP-level fragmentation, which can lead to performance degradation and increased packet loss. The protocol itself does not define a specific maximum record size, but implementations **MUST** be aware of the MTU of the underlying transport and network. When running over QUIC/UDP, the MTU is typically around 1200-1500 bytes, accounting for the overhead of IP, UDP, and QUIC headers. CrypRQ implementations **SHOULD** implement Path MTU Discovery (PMTUD) or a similar mechanism to determine the optimal record size for a given connection. If a record is too large to be sent in a single transport packet, it is the responsibility of the transport layer (e.g., QUIC) to handle fragmentation and reassembly. The CrypRQ record layer **MUST NOT** perform its own fragmentation. This ensures that the protocol remains simple and that the transport can optimize the delivery of data.

## 4. Handshake Protocol (ML-KEM + X25519 Hybrid)

### 4.1. Handshake Overview

The CrypRQ handshake is a critical component of the protocol, responsible for establishing a secure session between two peers. It is a hybrid handshake, meaning it combines two different key exchange mechanisms to achieve a high level of security that is resilient to both current and future threats. The handshake uses the post-quantum **ML-KEM (Kyber768)** key encapsulation mechanism and the classical **X25519** elliptic-curve Diffie-Hellman key agreement. This hybrid approach ensures that the session remains secure even if one of the underlying cryptographic primitives is broken. The handshake process involves the exchange of a series of messages between the Initiator and the Responder. These messages carry the public keys and ciphertexts required for the key exchange, as well as other negotiation parameters. The successful completion of the handshake results in a shared master secret, from which all subsequent application traffic keys are derived. The handshake is designed to be efficient, requiring only a few round trips, and to provide strong authentication and forward secrecy.

### 4.2. Handshake Message Flow

The CrypRQ v1.0 handshake consists of three messages:

*   `CRYPRQ_CLIENT_HELLO` (Initiator → Responder)
*   `CRYPRQ_SERVER_HELLO` (Responder → Initiator)
*   `CRYPRQ_CLIENT_FINISH` (Initiator → Responder)

This flow establishes a hybrid shared secret using ML-KEM (Kyber768) and X25519, from which a master secret and application traffic keys are derived.

#### 4.2.1. `CRYPRQ_CLIENT_HELLO`

The `CRYPRQ_CLIENT_HELLO` message is the first message sent by the Initiator. It is sent in plaintext and has the following structure:

*   **Version (1 byte):** Protocol version. For CrypRQ v1.0 this **MUST** be `0x01`.

*   **Random (32 bytes):** Cryptographically random value contributed by the Initiator to the key schedule and replay protection.

*   **Cipher Suites (variable length):** A vector of 2-byte cipher suite identifiers (e.g., AEAD algorithms). At least one cipher suite **MUST** be present.

*   **Extensions (variable length):** A TLV-encoded list of extensions. The format is:
    *   `ext_type` (2 bytes, big-endian)
    *   `ext_len` (2 bytes, big-endian)
    *   `ext_value` (`ext_len` bytes)

The `CRYPRQ_CLIENT_HELLO` does not carry key material; it announces capabilities and preferences.

#### 4.2.2. `CRYPRQ_SERVER_HELLO`

Upon receiving a valid `CRYPRQ_CLIENT_HELLO`, the Responder replies with `CRYPRQ_SERVER_HELLO`, also in plaintext:

*   **Version (1 byte):** **MUST** be equal to the client's proposed version (`0x01` for v1.0), otherwise the Responder **MUST** send an `ERROR` with `UNSUPPORTED_VERSION` and abort.

*   **Random (32 bytes):** Cryptographically random value contributed by the Responder.

*   **Cipher Suite (2 bytes):** The single AEAD cipher suite chosen from the client's list. If no overlap exists, the Responder **MUST** send an error and abort.

*   **ML-KEM Public Key (variable length):** Responder's ephemeral ML-KEM (Kyber768) public key, used by the Initiator to encapsulate a shared secret.

*   **X25519 Public Key (32 bytes):** Responder's ephemeral X25519 public key.

*   **Extensions (variable length):** A TLV list of extensions that the Responder accepts and will use for this session. Unrecognized extensions from the client are ignored.

At this point, the Responder has committed to its ephemeral key material and cipher suite.

#### 4.2.3. `CRYPRQ_CLIENT_FINISH`

After receiving `CRYPRQ_SERVER_HELLO`, the Initiator performs the hybrid key exchange and sends `CRYPRQ_CLIENT_FINISH`. This message is sent in plaintext but cryptographically authenticated via the `verify_data` field:

*   **ML-KEM Ciphertext (variable length):** Ciphertext produced by encapsulating to the Responder's ML-KEM public key from `CRYPRQ_SERVER_HELLO`.

*   **X25519 Public Key (32 bytes):** Initiator's ephemeral X25519 public key.

*   **Verify Data (variable length):** A MAC over the handshake transcript computed using a key derived from the hybrid shared secret (see Section 4.4).

This proves that the Initiator successfully computed the shared secrets and derived the master secret.

The handshake transcript includes, at minimum:

*   `CRYPRQ_CLIENT_HELLO` (as sent)
*   `CRYPRQ_SERVER_HELLO` (as sent)
*   All fields of `CRYPRQ_CLIENT_FINISH` except `verify_data`

Once the Responder receives `CRYPRQ_CLIENT_FINISH`, both sides compute the same hybrid master secret and verify `verify_data`. If verification fails, the Responder **MUST** send an `ERROR` with `CRYPTO_ERROR` and abort the session.

### 4.3. Hybrid Key Exchange

The CrypRQ handshake combines ML-KEM (Kyber768) and X25519 into a single hybrid shared secret.

#### 4.3.1. ML-KEM (Kyber768) Exchange

During `CRYPRQ_SERVER_HELLO`, the Responder:

*   Generates an ephemeral ML-KEM key pair (`kem_sk_r`, `kem_pk_r`).
*   Sends `kem_pk_r` as ML-KEM Public Key.

During `CRYPRQ_CLIENT_FINISH`, the Initiator:

*   Uses `kem_pk_r` to encapsulate:
    *   `(kem_ct, ss_kem_i) = MLKEM_Encaps(kem_pk_r)`
*   Sends `kem_ct` as ML-KEM Ciphertext.

Upon receiving `CRYPRQ_CLIENT_FINISH`, the Responder:

*   Computes `ss_kem_r = MLKEM_Decaps(kem_sk_r, kem_ct)`.

Both peers **MUST** obtain the same ML-KEM shared secret:

*   `ss_kem = ss_kem_i = ss_kem_r`

#### 4.3.2. X25519 Exchange

The Responder generates an ephemeral X25519 key pair (`x_sk_r`, `x_pk_r`) and sends `x_pk_r` in `CRYPRQ_SERVER_HELLO`.

The Initiator generates an ephemeral X25519 key pair (`x_sk_i`, `x_pk_i`) and sends `x_pk_i` in `CRYPRQ_CLIENT_FINISH`.

Each side computes:

*   Initiator: `ss_x_i = X25519(x_sk_i, x_pk_r)`
*   Responder: `ss_x_r = X25519(x_sk_r, x_pk_i)`

Both **MUST** obtain the same X25519 shared secret:

*   `ss_x = ss_x_i = ss_x_r`

#### 4.3.3. Hybrid Shared Secret

The two shared secrets are combined:

*   `ss_kem` (from ML-KEM)
*   `ss_x` (from X25519)

The input keying material (IKM) to HKDF is:

*   `ikm = ss_kem || ss_x`

This IKM is fed into the key schedule described in Section 4.4 to produce the master secret and subsequent traffic keys.

### 4.4. Key Schedule

After the Initiator sends `CRYPRQ_CLIENT_FINISH` and the Responder processes it, both peers know `ss_kem` and `ss_x`. From these, the key schedule derives:

*   A handshake authentication key for computing and verifying `verify_data`.
*   A master secret for application traffic keys.

A concrete split that preserves the original semantics:

```
salt_hs = "cryp-rq v1.0 hs"
ikm     = ss_kem || ss_x

prk_hs        = HKDF-Extract(salt_hs, ikm)
hs_auth_key   = HKDF-Expand(prk_hs, "cryp-rq hs auth", L_auth)
master_secret = HKDF-Expand(prk_hs, "cryp-rq master secret", 32)
```

Then:

```
verify_data = HMAC(hs_auth_key, handshake_transcript)
```

Application traffic keys are derived from `master_secret` as described in Section 4.4.2 (Initiator→Responder / Responder→Initiator keys and IVs).

#### 4.4.1. Master Secret Derivation

The master secret (`MS`) is derived from the two shared secrets (`ss_kem` and `ss_x`) using HKDF. The process is as follows:

1.  **Extract:** `PRK = HKDF-Extract(salt_hs, ss_kem || ss_x)`

    *   `salt_hs = "cryp-rq v1.0 hs"` is the salt value.

    *   `||` denotes concatenation.

    *   `PRK` is the pseudorandom key.

2.  **Expand:** `MS = HKDF-Expand(PRK, "cryp-rq master secret", 32)`

    *   `"cryp-rq master secret"` is a context-specific label.

    *   The master secret length is 32 bytes.

#### 4.4.2. Application Traffic Key Derivation

From the master secret, two sets of application traffic keys are derived: one for traffic from the Initiator to the Responder, and one for traffic from the Responder to the Initiator. Each set consists of an encryption key and an IV (initialization vector) for the AEAD cipher. The derivation process is:

1.  **Initiator-to-Responder Key:** `key_ir = HKDF-Expand(MS, "cryp-rq ir key", L_key)`

2.  **Initiator-to-Responder IV:** `iv_ir = HKDF-Expand(MS, "cryp-rq ir iv", L_iv)`

3.  **Responder-to-Initiator Key:** `key_ri = HKDF-Expand(MS, "cryp-rq ri key", L_key)`

4.  **Responder-to-Initiator IV:** `iv_ri = HKDF-Expand(MS, "cryp-rq ri iv", L_iv)`

The lengths `L_key` and `L_iv` depend on the selected AEAD cipher suite.

#### 4.4.3. Key Derivation Pseudocode

The following pseudocode illustrates the key derivation process:

```
function derive_master_secret(ss_kem, ss_x):
    salt_hs = "cryp-rq v1.0 hs"
    ikm = ss_kem || ss_x
    prk_hs = HKDF-Extract(salt_hs, ikm)
    hs_auth_key = HKDF-Expand(prk_hs, "cryp-rq hs auth", L_auth)
    master_secret = HKDF-Expand(prk_hs, "cryp-rq master secret", 32)
    return (hs_auth_key, master_secret)

function derive_traffic_keys(master_secret):
    key_ir = HKDF-Expand(master_secret, "cryp-rq ir key", 32)  // For AES-256-GCM
    iv_ir = HKDF-Expand(master_secret, "cryp-rq ir iv", 12)    // For AES-256-GCM
    key_ri = HKDF-Expand(master_secret, "cryp-rq ri key", 32)
    iv_ri = HKDF-Expand(master_secret, "cryp-rq ri iv", 12)
    return (key_ir, iv_ir, key_ri, iv_ri)
```

### 4.5. Peer Authentication

CrypRQ v1.0 provides entity authentication by binding the handshake transcript to long-term identity keys.

Each peer is assumed to possess a long-term identity key pair. The concrete identity scheme is deployment-specific and **MAY** be:

*   A raw Ed25519 public key
*   An X.509 certificate chain
*   A libp2p-style peer ID
*   A pre-shared key (PSK) bound to an abstract identity

Identity material is carried in a handshake extension (e.g., identity extension) in `CRYPRQ_CLIENT_HELLO` and/or `CRYPRQ_SERVER_HELLO`.

For each peer:

*   The peer presents its identity (e.g., certificate or public key) in a handshake extension.
*   The peer computes and sends `verify_data` in `CRYPRQ_CLIENT_FINISH`, which is a MAC over the handshake transcript using `hs_auth_key` derived from the hybrid shared secret (Section 4.4).

The other side:

*   Validates the presented identity according to local policy (certificate validation, pinned key, configured trust store, etc.).
*   Recomputes `verify_data` and checks for equality.

If identity verification or `verify_data` verification fails, the peer **MUST** send an `ERROR` with `CRYPTO_ERROR` and abort the session.

This provides mutual authentication at the protocol level, assuming the deployment correctly manages identity keys and trust anchors.

## 5. Key Schedule and Rotation

### 5.1. Initial Application Keys

Upon successful completion of the handshake, both peers derive the initial set of application traffic keys. As described in the previous section, these keys are derived from the master secret using a KDF. There are two distinct sets of keys: one for encrypting data sent from the Initiator to the Responder (`key_ir`, `iv_ir`), and another for data sent from the Responder to the Initiator (`key_ri`, `iv_ri`). This separation of keys for each direction is a crucial security feature, as it prevents an attacker who compromises one direction from decrypting traffic in the other. These initial keys are used to secure all application data until the first key rotation event occurs. The derivation of these keys is a one-time event at the beginning of the session, and their security is paramount to the overall security of the communication.

### 5.2. Directional Keys and Nonces

From `master_secret`, CrypRQ derives two independent sets of application traffic keys:

*   Initiator → Responder: `key_ir`, `iv_ir`
*   Responder → Initiator: `key_ri`, `iv_ri`

This separation ensures that compromising one direction does not automatically compromise the other.

#### 5.2.1. Sequence Numbers

Each peer maintains a 64-bit sequence number per sending direction. The sequence number:

*   Starts at 0 when application keys for that direction are first installed.
*   Is incremented by 1 for each record sent with that key.
*   **MUST NOT** wrap; if it reaches 2^64 - 1, the implementation **MUST** trigger a key update or terminate the session.

The sequence number is transmitted in the record header (Section 6.1) and is used for both replay protection and AEAD nonce construction.

#### 5.2.2. Nonce Construction

CrypRQ follows a TLS 1.3–style nonce construction for AEAD algorithms with a 96-bit nonce (e.g., AES-GCM, ChaCha20-Poly1305):

For each direction:

*   `static_iv` – 96-bit IV derived from the key schedule for that direction.
*   `seq` – 64-bit sequence number for this record.

To form the nonce:

1.  Encode `seq` as 96-bit big-endian:
    *   `seq_be = 0x00000000 || seq_64_be`  // 32 zero bits + 64-bit big-endian seq

2.  Compute:
    *   `nonce = static_iv XOR seq_be`

This construction guarantees a unique nonce for every record under a given key, as long as the sequence number does not wrap, and is compatible with standard AEAD libraries.

The entire record header is used as AEAD Associated Data (AAD) (see Section 6.2).

### 5.3. Ephemeral Key Rotation

CrypRQ supports periodic rotation of application traffic keys to limit the amount of data protected under a single key and to improve forward secrecy.

#### 5.3.1. Epoch Management

Each session maintains an epoch field that identifies the generation of traffic keys in use. In CrypRQ v1.0:

*   The epoch is an 8-bit unsigned integer transmitted in the record header.
*   The initial epoch value is 0.
*   On each successful key rotation, the epoch value is incremented modulo 256.

Implementations **SHOULD** ensure that sessions are not kept alive long enough to exhaust all 256 epochs with the same master secret. Long-lived deployments **SHOULD** initiate a new session before epoch exhaustion.

The current epoch value is included in every record header, allowing the receiver to select the correct keys.

#### 5.3.2. Key Update Process

A key update is triggered by one peer (the initiator of the update):

*   The updating peer increments its local epoch:
    *   `epoch := (epoch + 1) mod 256`

*   Using the same `master_secret`, the peer derives a new set of directional keys and IVs scoped by the epoch:
    *   `key_ir = HKDF-Expand(master_secret, "cryp-rq ir key epoch=" || epoch, L_key)`
    *   `iv_ir  = HKDF-Expand(master_secret, "cryp-rq ir iv epoch="  || epoch, L_iv)`
    *   `key_ri = HKDF-Expand(master_secret, "cryp-rq ri key epoch=" || epoch, L_key)`
    *   `iv_ri  = HKDF-Expand(master_secret, "cryp-rq ri iv epoch="  || epoch, L_iv)`

*   The peer sends a `KEY_UPDATE` control message (Section 7.7) to the other side, indicating the new epoch value.

Upon receiving `KEY_UPDATE`, the other peer:

*   Updates its epoch to the indicated value.
*   Derives the same epoch-scoped keys.
*   Starts using them for subsequent records.

Old keys **SHOULD** be retained for a short grace period to allow decryption of in-flight records from the previous epoch, after which they **MUST** be securely erased.

#### 5.3.3. Handling Out-of-Sync Epochs

If a peer receives a record with:

*   **An epoch higher than its current epoch:**
    *   It **MUST** derive keys for that epoch (using the same derivation as above) and update its local epoch.
    *   It **MAY** also expect that a `KEY_UPDATE` message was missed.

*   **An epoch lower than its current epoch:**
    *   It **MAY** attempt decryption with the old keys if they are still retained.
    *   If decryption fails or the epoch is outside the retention window, the record **MUST** be discarded.

If epochs diverge in a way that cannot be reconciled (e.g., multiple jumps, inconsistent behavior), the implementation **MAY** send an `ERROR` and terminate the session.

## 6. Record Layer and Wire Format

### 6.1. Record Structure

All application data and control messages are carried in CrypRQ records. Each record consists of a fixed-size header followed by an AEAD ciphertext payload.

#### 6.1.1. Header Layout

The CrypRQ v1.0 record header is 20 bytes and has the following fields, in order:

*   **Version (1 byte):** Protocol version (`0x01` for v1.0).

*   **Message Type (1 byte):** See the Message Type Registry (Section 7.1).

*   **Flags (1 byte):** Bitfield for message-specific options (semantics depend on message type).

*   **Epoch (1 byte):** 8-bit epoch value used for key rotation (Section 5.3).

*   **Stream ID (4 bytes, big-endian):** 32-bit identifier of the logical stream.

*   **Sequence Number (8 bytes, big-endian):** 64-bit monotonically increasing sequence number for this direction and stream.

*   **Ciphertext Length (4 bytes, big-endian):** Length of the ciphertext payload in bytes.

A conceptual byte diagram:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Ver | Type  |  Flags | Epoch|          Stream ID           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Stream ID (cont.)                    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       Sequence Number (high)                 |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       Sequence Number (low)                  |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                     Ciphertext Length                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Ciphertext ...                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

The precise bit packing of Version, Type, Flags, and Epoch is byte-aligned; they are four consecutive single-byte fields.

#### 6.1.2. Ciphertext Payload

The ciphertext payload is produced by applying the chosen AEAD algorithm to:

*   The plaintext payload (message-type specific), and
*   The header bytes as Associated Data (AAD), using the direction-appropriate key and nonce (Sections 5.2 and 6.2).

The Ciphertext Length field specifies the length in bytes of this AEAD output (ciphertext + authentication tag, depending on AEAD implementation).

### 6.2. AEAD Encryption and Decryption

CrypRQ uses Authenticated Encryption with Associated Data (AEAD) ciphers to provide both confidentiality and integrity. The encryption and decryption processes are as follows:

#### 6.2.1. Associated Data (AAD)

The AEAD algorithm takes three inputs: a key, a nonce, and the plaintext. It also takes a fourth input, the **Associated Data (AAD)** , which is authenticated but not encrypted. In CrypRQ, the AAD consists of the entire record header (Version, Message Type, Flags, Epoch, Stream ID, Sequence Number, and Ciphertext Length). This ensures that the header cannot be tampered with without being detected. The AAD is passed to the AEAD encryption and decryption functions, but it is not included in the ciphertext output.

#### 6.2.2. Nonce Construction

CrypRQ uses AEAD algorithms with a 96-bit nonce (e.g., AES-GCM, ChaCha20-Poly1305). For each direction (Initiator→Responder and Responder→Initiator), the key schedule provides:

*   `static_iv` – a 96-bit IV unique to that direction and epoch.
*   A 64-bit sequence number `seq` per record.

To construct the nonce for a record:

1.  Encode the 64-bit sequence number as 96-bit big-endian:
    *   `seq_be = 0x00000000 || seq_64_be`

2.  Compute the nonce as:
    *   `nonce = static_iv XOR seq_be`

This ensures that each record uses a unique nonce under a given key, as long as sequence numbers do not wrap, and matches the well-understood pattern used in TLS 1.3.

Implementations **MUST NOT** re-use a (key, nonce) pair. If sequence number exhaustion is imminent, a key update or session restart **MUST** occur before sending further records.

### 6.3. Example Record Encoding

The following is an example of a CrypRQ record, shown in hexadecimal format, with annotations:

```
Offset:  0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
0000:   01 01 00 00  00 00 00 01  00 00 00 00  00 00 00 01
0010:   00 00 00 2C  5C 3F  ... (ciphertext) ...
```

*   **Byte 0 (`0x01`):** Version (v1.0)

*   **Byte 1 (`0x01`):** Message Type (`DATA`)

*   **Byte 2 (`0x00`):** Flags (none set)

*   **Byte 3 (`0x00`):** Epoch (0)

*   **Bytes 4-7 (`0x00000001`):** Stream ID (1)

*   **Bytes 8-15 (`0x0000000000000001`):** Sequence Number (1)

*   **Bytes 16-19 (`0x0000002C`):** Ciphertext Length (44 bytes)

*   **Bytes 20+:** The AEAD-encrypted ciphertext payload.

This example shows a `DATA` message being sent on stream 1, with sequence number 1, and a payload of 44 bytes. The record is encrypted using the traffic keys for epoch 0.

## 7. Message Types and Semantics

### 7.1. Message Type Registry

The following table defines the message types used in the CrypRQ protocol. Each message type is identified by a unique numeric code.

| Code | Name         | Description                                                                 |
| :--- | :----------- | :-------------------------------------------------------------------------- |
| 0x01 | `DATA`       | Generic stream/tunnel data.                                                 |
| 0x02 | `FILE_META`  | Metadata for a file transfer (name, size, hash, transfer ID).               |
| 0x03 | `FILE_CHUNK` | A chunk of file data.                                                       |
| 0x04 | `FILE_ACK`   | Acknowledgment for received file chunks or ranges.                          |
| 0x05 | `VPN_PACKET` | A raw IP packet for VPN/TUN mode.                                           |
| 0x10 | `CONTROL`    | Control messages (e.g., ping, close, error, keepalive, key update).         |
| 0xFF | `RESERVED`   | Reserved for future use.                                                    |

### 7.2. `DATA` Message

The `DATA` message is used to carry arbitrary application data over a CrypRQ stream. It is the most common type of message. The payload of a `DATA` message is simply a sequence of bytes provided by the application. There is no specific structure for the payload. The receiver of a `DATA` message **MUST** deliver the payload to the application-level handler for the corresponding stream. The `DATA` message has no specific flags defined.

### 7.3. `FILE_META` Message

The `FILE_META` message is the first message sent in a file transfer session. It contains the metadata for the file being transferred. The payload of a `FILE_META` message is a structured object, which can be encoded in a format like JSON or Protocol Buffers. The fields of this object **MUST** include:

*   **File Name:** The name of the file.

*   **File Size:** The total size of the file in bytes.

*   **SHA-256 Hash:** The SHA-256 hash of the entire file, used for integrity verification at the end of the transfer.

*   **Transfer ID:** A unique identifier for this file transfer session.

Upon receiving a `FILE_META` message, the receiver **MUST** create a new file transfer session and prepare to receive the file data.

### 7.4. `FILE_CHUNK` Message

The `FILE_CHUNK` message is used to send a portion of the file data. The payload of a `FILE_CHUNK` message consists of a chunk of the file's bytes. The size of the chunk is determined by the sender and can be variable. The receiver of a `FILE_CHUNK` message **MUST** write the chunk of data to the appropriate location in the file, based on the offset information that may be included in the message or implied by the sequence of chunks. The receiver **SHOULD** send a `FILE_ACK` message to acknowledge the receipt of the chunk.

### 7.5. `FILE_ACK` Message

The `FILE_ACK` message is used to acknowledge the successful receipt of file chunks. The payload of a `FILE_ACK` message can be a simple acknowledgment of the last received chunk, or it can be a more complex structure that acknowledges a range of chunks. This allows the sender to track the progress of the transfer and to retransmit any lost chunks. The `FILE_ACK` message is crucial for ensuring the reliability of the file transfer.

### 7.6. `VPN_PACKET` Message

The `VPN_PACKET` message is used in VPN mode to transmit raw IP packets. The payload of a `VPN_PACKET` message is a single IP packet, as captured from the TUN interface. The receiver of a `VPN_PACKET` message **MUST** write the packet to its TUN interface, which will then process it as if it had been received from a physical network interface. This allows the two peers to form a virtual network link.

### 7.7. `CONTROL` Message

The `CONTROL` message is used for various control and management functions. The payload of a `CONTROL` message is a structured object that includes a control message type and any associated parameters. The following control message types are defined:

*   **PING:** A simple heartbeat message to check if the peer is still alive.

*   **PONG:** The response to a `PING` message.

*   **CLOSE:** A message to gracefully close a stream or the entire session.

*   **ERROR:** A message to signal an error condition.

*   **KEY_UPDATE:** A message to initiate a key rotation.

*   **KEEPALIVE:** A message to keep the session alive when no data is being sent.

The `CONTROL` message is essential for managing the state of the connection and for handling error conditions.

## 8. Error Handling

### 8.1. Error Code Registry

The following table defines the error codes used in the CrypRQ protocol. These codes are used in `ERROR` control messages to signal specific error conditions.

| Code | Name                 | Description                                                                 |
| :--- | :------------------- | :-------------------------------------------------------------------------- |
| 0x01 | `PROTOCOL_VIOLATION` | A general protocol violation has occurred.                                  |
| 0x02 | `UNEXPECTED_MESSAGE` | A message was received that was not expected in the current state.          |
| 0x03 | `CRYPTO_ERROR`       | A cryptographic error has occurred (e.g., decryption failure).              |
| 0x04 | `STREAM_CLOSED`      | An operation was attempted on a stream that has already been closed.        |
| 0x05 | `UNSUPPORTED_VERSION`| The peer does not support the protocol version requested in the handshake.  |

### 8.2. Signaling Errors

When a peer encounters an error, it **SHOULD** send an `ERROR` control message to the other peer. The payload of the `ERROR` message **MUST** include the error code and a human-readable error message. The error message is for debugging purposes and is not intended to be parsed by the receiver. The `ERROR` message allows the peer that caused the error to be notified of the problem, which can help in diagnosing and resolving issues.

### 8.3. Session Termination Rules

Upon receiving an `ERROR` message, the receiving peer **MUST** take appropriate action based on the error code. For some errors, such as `UNSUPPORTED_VERSION`, the peer **MAY** choose to terminate the session immediately. For other errors, such as `UNEXPECTED_MESSAGE`, the peer **MAY** choose to ignore the error and continue the session, or it **MAY** choose to terminate the session. If a peer receives a `CLOSE` message, it **MUST** gracefully terminate the specified stream or the entire session, depending on the parameters of the message. In all cases, when a session is terminated, all associated state, including cryptographic keys, **MUST** be securely erased from memory.

## 9. Security Considerations

### 9.1. Threat Model

The CrypRQ protocol is designed to provide security in the presence of a powerful adversary. The threat model assumes that an attacker can:

*   **Observe** all traffic between the two peers.

*   **Drop, delay, or reorder** packets.

*   **Inject** new packets into the communication stream.

*   **Modify** the contents of packets in transit.

The protocol is designed to protect against these attacks by providing confidentiality, integrity, and authenticity. However, the threat model does not assume that the attacker can break the underlying cryptographic primitives (ML-KEM, X25519, AEAD, SHA-256). The security of the protocol relies on the strength of these algorithms.

### 9.2. Security Goals

The primary security goals of the CrypRQ protocol are:

*   **Confidentiality:** Ensuring that all application data is encrypted and cannot be read by an unauthorized third party.

*   **Integrity:** Ensuring that all data is authenticated and any tampering is detected.

*   **Authenticity:** Ensuring that the communicating peers are who they claim to be.

*   **Forward Secrecy:** Ensuring that the compromise of long-term keys does not compromise past session keys. This is achieved through the use of ephemeral keys and periodic key rotation.

*   **Post-Quantum Security:** Ensuring that the session remains secure even in the face of attacks from quantum computers. This is achieved through the use of the ML-KEM (Kyber768) key encapsulation mechanism.

### 9.3. Non-Goals and Limitations

It is important to understand the limitations of the CrypRQ protocol. As stated in the introduction, the protocol does not provide:

*   **Anonymity:** The protocol does not hide the IP addresses of the communicating peers.

*   **Traffic Analysis Resistance:** The protocol does not obscure the size, timing, or frequency of packets.

*   **Censorship Resistance:** The protocol's traffic patterns may be identifiable by deep packet inspection systems.

These are considered out of scope for v1.0 of the protocol.

### 9.4. Replay Resistance

The protocol provides protection against replay attacks through the use of sequence numbers and nonces. Each record is assigned a unique, monotonically increasing sequence number. This sequence number is included in the AAD for the AEAD encryption, which means that any attempt to replay an old record will be detected, as the sequence number will not match the expected value. Furthermore, the sequence number is used in the construction of the nonce for the AEAD cipher. Since the nonce must be unique for each encryption operation, a replayed record will have an incorrect nonce, and the decryption will fail.

### 9.5. Random Number Generation

The security of the entire protocol depends on the quality of the random number generator (RNG) used by the implementation. All random values, including the random values in the handshake messages, the ephemeral private keys, and the nonces, **MUST** be generated using a cryptographically secure RNG. A weak or predictable RNG can completely compromise the security of the protocol. Implementations **MUST** use a well-vetted RNG, such as the one provided by the operating system (e.g., `/dev/urandom` on Unix-like systems).

## 10. Versioning and Extensibility

### 10.1. Version Negotiation

The CrypRQ protocol includes a simple version negotiation mechanism. The version of the protocol is included in the `CRYPRQ_CLIENT_HELLO` and `CRYPRQ_SERVER_HELLO` messages. The Initiator proposes a version, and the Responder either accepts it by echoing the same version in its `CRYPRQ_SERVER_HELLO` message, or it rejects it by sending an `ERROR` message with the `UNSUPPORTED_VERSION` code. If the Responder supports multiple versions, it can choose the highest version that is also supported by the Initiator. This mechanism allows for a straightforward negotiation of the protocol version to be used for the session.

### 10.2. Extensions Mechanism

The protocol includes an extensions mechanism to allow for the addition of new features without breaking compatibility with older implementations. Extensions are negotiated during the handshake. The Initiator includes a list of the extensions it supports in its `CRYPRQ_CLIENT_HELLO` message. The Responder then includes a list of the extensions it agrees to use in its `CRYPRQ_SERVER_HELLO` message. The format of the extensions list is a TLV (Type-Length-Value) encoding. If a peer receives an extension that it does not understand, it **MUST** ignore it. This allows for forward compatibility, as new extensions can be added to the protocol without causing older implementations to fail.

### 10.3. IANA-Style Registry

To manage the allocation of identifiers for message types, extensions, and error codes, this specification defines a simple registry. This is not a formal IANA registry, but it serves a similar purpose within the context of the CrypRQ protocol. The registries are as follows:

*   **Message Type Registry:** This registry contains the list of defined message types and their numeric codes.

*   **Extension Registry:** This registry contains the list of defined extensions and their numeric codes.

*   **Error Code Registry:** This registry contains the list of defined error codes and their numeric codes.

The ranges for these registries are as follows:

*   **0x00 - 0x0F:** Reserved for core protocol messages.

*   **0x10 - 0x7F:** Available for assignment for standard extensions and message types.

*   **0x80 - 0xFE:** Reserved for private or experimental use.

*   **0xFF:** Reserved for future use.

## 11. Operational Considerations (Non-Normative)

### 11.1. Timeouts and Retries

Implementations **SHOULD** use reasonable timeouts for all network operations. For example, a timeout for the handshake process should be long enough to account for network latency, but not so long that it allows for a denial-of-service attack. Similarly, timeouts for control messages like `PING` should be used to detect dead peers. If a message is not acknowledged within a certain time, the implementation **MAY** choose to retransmit it. However, the retransmission logic should be implemented with care to avoid congestion. The use of an exponential backoff algorithm for retransmissions is recommended.

### 11.2. Logging Best Practices

Implementations **SHOULD** provide a logging mechanism to aid in debugging and monitoring. However, care must be taken to avoid logging sensitive information. The following information **SHOULD NOT** be logged:

*   Cryptographic keys (private keys, shared secrets, session keys).

*   The plaintext of application data.

*   Any information that could be used to identify a user or their activities.

The following information **MAY** be logged:

*   The establishment and termination of sessions.

*   The negotiation of cryptographic parameters.

*   The occurrence of errors.

*   High-level statistics about the connection (e.g., bytes transferred, duration).

Logs should be structured and machine-readable to facilitate analysis.

### 11.3. Deployment Patterns

CrypRQ is a versatile protocol that can be deployed in a variety of ways. Some common deployment patterns include:

*   **Web-Only Mode:** In this mode, the protocol is used to provide a secure tunnel for web-based applications. This is the default mode for the reference implementation.

*   **VPN Mode:** In this mode, the protocol is used to create a system-wide VPN. This requires the creation of a TUN interface and the configuration of routing rules.

*   **File-Transfer-Only Mode:** In this mode, the protocol is used solely for the purpose of transferring files. This can be useful in scenarios where a full VPN is not needed.

The choice of deployment pattern will depend on the specific use case and the requirements of the application.

## 12. Appendices

### 12.1. Appendix A: Example Handshake Trace

The following is a simplified example of a CrypRQ v1.0 handshake, showing how the hybrid shared secret and master secret are derived. This is illustrative, not normative pseudocode.

```
// 1. Initiator (client) state
client_x_sk, client_x_pk     = X25519_GenerateKeyPair()
client_random                = Random(32)
client_cipher_suites         = [CHACHA20_POLY1305, AES_256_GCM]

// 2. Send CRYPRQ_CLIENT_HELLO (plaintext)
send(CRYPRQ_CLIENT_HELLO {
    version       = 0x01,
    random        = client_random,
    cipher_suites = client_cipher_suites,
    extensions    = [...]
})

// 3. Responder (server) state
server_x_sk, server_x_pk     = X25519_GenerateKeyPair()
server_kem_sk, server_kem_pk = MLKEM_GenerateKeyPair()
server_random                = Random(32)
chosen_cipher_suite          = CHACHA20_POLY1305

// 4. Send CRYPRQ_SERVER_HELLO (plaintext)
send(CRYPRQ_SERVER_HELLO {
    version      = 0x01,
    random       = server_random,
    cipher_suite = chosen_cipher_suite,
    kem_pk       = server_kem_pk,
    x25519_pk    = server_x_pk,
    extensions   = [...]
})

// 5. Client computes hybrid secrets
// from server_x_pk and server_kem_pk

// X25519 shared secret
ss_x_client  = X25519(client_x_sk, server_x_pk)

// ML-KEM encapsulation
kem_ct, ss_kem_client = MLKEM_Encaps(server_kem_pk)

// Concatenate secrets for HKDF input
ikm = ss_kem_client || ss_x_client

// Derive handshake auth key + master_secret
salt_hs        = "cryp-rq v1.0 hs"
prk_hs         = HKDF-Extract(salt_hs, ikm)
hs_auth_key    = HKDF-Expand(prk_hs, "cryp-rq hs auth", L_auth)
master_secret  = HKDF-Expand(prk_hs, "cryp-rq master secret", 32)

// Build handshake transcript (excluding verify_data)
transcript = concat(
    serialize(CRYPRQ_CLIENT_HELLO),
    serialize(CRYPRQ_SERVER_HELLO),
    serialize(CRYPRQ_CLIENT_FINISH without verify_data)
)

// Compute verify_data
verify_data = HMAC(hs_auth_key, transcript)

// 6. Client sends CRYPRQ_CLIENT_FINISH (plaintext)
send(CRYPRQ_CLIENT_FINISH {
    kem_ct     = kem_ct,
    x25519_pk  = client_x_pk,
    verify_data = verify_data
})

// 7. Server receives CRYPRQ_CLIENT_FINISH and computes secrets

// X25519 shared secret
ss_x_server  = X25519(server_x_sk, client_x_pk)

// ML-KEM decapsulation
ss_kem_server = MLKEM_Decaps(server_kem_sk, kem_ct)

// Both must match:
assert(ss_x_server  == ss_x_client)
assert(ss_kem_server == ss_kem_client)

// Derive same keys
ikm        = ss_kem_server || ss_x_server
prk_hs     = HKDF-Extract(salt_hs, ikm)
hs_auth_key= HKDF-Expand(prk_hs, "cryp-rq hs auth", L_auth)
master_secret = HKDF-Expand(prk_hs, "cryp-rq master secret", 32)

// Rebuild transcript and verify verify_data
transcript = concat(
    serialize(CRYPRQ_CLIENT_HELLO),
    serialize(CRYPRQ_SERVER_HELLO),
    serialize(CRYPRQ_CLIENT_FINISH without verify_data)
)

expected_verify_data = HMAC(hs_auth_key, transcript)
if verify_data != expected_verify_data:
    abort(CRYPTO_ERROR)

// 8. Both sides now derive application traffic keys
(key_ir, iv_ir, key_ri, iv_ri) = derive_traffic_keys(master_secret)

// Handshake complete: application records can now be sent.
```

### 12.2. Appendix B: Example Record Encodings

This appendix provides several example CrypRQ record encodings in hexadecimal format.

**Example 1: `DATA` Message**

```
01 01 00 00  00 00 00 01  00 00 00 00  00 00 00 01
00 00 00 0A  3C 7F  ... (10 bytes of ciphertext) ...
```

*   **Version:** 0x01 (v1.0)
*   **Message Type:** 0x01 (`DATA`)
*   **Flags:** 0x00
*   **Epoch:** 0x00
*   **Stream ID:** 0x00000001 (1)
*   **Sequence Number:** 0x0000000000000001 (1)
*   **Ciphertext Length:** 0x0000000A (10 bytes)

**Example 2: `FILE_META` Message**

```
01 02 00 00  00 00 00 02  00 00 00 00  00 00 00 01
00 00 00 50  9A B2  ... (80 bytes of ciphertext) ...
```

*   **Version:** 0x01 (v1.0)
*   **Message Type:** 0x02 (`FILE_META`)
*   **Flags:** 0x00
*   **Epoch:** 0x00
*   **Stream ID:** 0x00000002 (2)
*   **Sequence Number:** 0x0000000000000001 (1)
*   **Ciphertext Length:** 0x00000050 (80 bytes)

### 12.3. Appendix C: Test Vector Guidelines

This appendix provides guidelines for the structure of test vectors for the CrypRQ protocol. Test vectors are essential for ensuring the correctness and interoperability of implementations. A test vector for CrypRQ should include the following information:

*   **Protocol Version:** The version of the protocol being tested.

*   **Handshake Messages:** The full, hexadecimal-encoded handshake messages (`CRYPRQ_CLIENT_HELLO`, `CRYPRQ_SERVER_HELLO`, `CRYPRQ_CLIENT_FINISH`).

*   **Ephemeral Keys:** The private and public keys used for the X25519 and ML-KEM key exchanges.

*   **Shared Secrets:** The intermediate shared secrets (`ss_x25519`, `ss_kem`) and the final master secret.

*   **Traffic Keys:** The derived application traffic keys (e.g., `client_write_key`, `server_write_key`).

*   **Record Examples:** A set of example records, including the plaintext, the associated data (AAD), the nonce, the key, and the resulting ciphertext.

By providing a comprehensive set of test vectors, implementers can verify that their code correctly implements all aspects of the protocol, from the handshake to the record layer.
