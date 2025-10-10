# Summary

This proposal aims to introduce a novel mechanism of permissioned tokens without the drawbacks of the existing solutions. By following this specification, issuers can create permissioned tokens using Token22, the Default Account State extension and an allow/block listing smart contract with delegated freezing authority.

# Context

Permissioned tokens fall into one of three use cases:

1. As an issuer I want to block X users
2. As an issuer I want to allow Y users
3. As an issuer I need to execute some custom logic in order to allow Z users to transact

This proposal targets use cases 1 and 2, these are the permissioning happy-paths that should have better UX without compromising performance.

# Background

Permissioned tokens in solana, before Token22, were based on wrapper programs that would thaw/freeze token accounts during each user interaction, at the cost of UX.

Token22 aimed to introduce alternatives while maintaining UX. The transfer-hook extension has a standardized interface that enables everyone to transfer and still execute custom code without requiring specialized UIs.

Even though this fixes the wallet UX this comes with great cost to protocol developers as it adds friction in the form of overhead compute units used during transfers and account dependency hell. This complexity leads most protocols simply blacklisting all token Mints with the transfer-hook extension.

Alternatively, issuers can use the Default Account State (DAS) extension to create permissioned tokens. This alternative trades UX for DX. Developer experience and composability are maintained, but user experience becomes significantly degraded. Token holders require the issuers manual intervention to thaw their token accounts before interacting with protocols. The issuers need to constantly thaw token accounts for their users, this is specially bothersome when related to sanctions lists where issuers only care about blocking some users.

# Proposal

A new mechanism that borrows experience from previous methods and uses the DAS extension along with a Smart Contract (hereon referred to by Token ACL) delegated freeze authority. Additionally a second, user defined, Smart Contract - Gate Program - that implements a specific interface with instructions that gate the ability of the previous one from permissionlessly calling the freeze and thaw functions on Token22 for a given Token Account. This approach borrows from the widely controversial transfer-hook workflow without compromising token transfer user and developer experience - it only checks whether the Token ACL should be able to permissionlessly thaw or freeze a TA.

The novelty in this workflow is the removed issuer friction of having to manually thaw every single token account without sacrificing composability and transaction compute usage for most allow/block listing scenarios. The only assumption is that there may be some on-chain record that enables the thawing gating business logic to allow or block a given wallet from permissionlessly thawing their TAs.

Additionally, with the freeze authority, it’s easy for issuers to revoke authorization anytime by freezing a user’s token account.

The Token ACL will have a canonical implementation (like the Token and Token22 programs), issuers who want to use this mechanism only need to write or use a single Smart Contract that implements the interface. When using the interface, the implementation decides whether a given method is supported or not, and how to behave if not supported - always accept or fail these instruction calls.

The Token ACL will still allow a regular defined Freeze Authority that is kept under control of the issuer and the Gate Program that gates the permissionless functions only has the ability to fail those transactions. This means that issuers can use a 3rd party created allow or block list and still remain in full control of their authorities without compromising any other functionality or authority.

# Specification

## Token Program

This standard requires a Token22 based token as it depends on the Default Account State extension.

The token needs to delegate the Freeze Authority to the Token ACL described in the next section.

## Token ACL

The Token ACL is a smart contract that augments the capabilities of the freeze authority for a given token. This new program not only maintains the ability to freeze and thaw tokens using an issuer defined freeze authority but this also introduces the capability for permissionless thawing and permissionless freezing of token accounts by using an issuer defined Smart Contract with gating business rules.

In order for the Token ACL to work, it requires that issuers delegate their freeze authority over. Given that freezing and thawing is such an important part of RWA workflows, the program maintains the same baseline features.

The new permissionless features are a means for anyone to be able to thaw or freeze token accounts when issuers use DAS extension on Token22 Token mints. These new permissionless instructions will call certain functions of a freeze authority defined Smart Contract that is responsible for deciding whether a Token Account should be frozen or thawed.

Using either the permissionless thaw and/or the permissionless freeze should be optional and defined by the freeze authority. This enables greater flexibility and allows the Gate Program to be an allow or block list operated by a 3rd party independent of the token issuer and freeze authority.

In order to maintain a secure environment, the Token ACL should ensure that permissionless instructions de-escalate account permissions when calling into the Gate Program code to prevent abuse from bad actors.

### Accounts

**MintConfig**

Is a PDA that stores configurations and is going to be the delegated freeze authority for a given token mint.

PDA derivation: [b“MINT_CFG”, mint_address]

Discriminator: u8 = 0x01

Structure:

- mint: Pubkey
    - The mint this MintConfig is associated with. Even though this could be handled by PDA derivation, it’s easier for fetching and discovery.

- authority: Pubkey
    - User defined authority capable of changing the gating program and calling the permissioned instructions

- gating_program: Pubkey
    - Gate program. Pubkey::default() for none.

- enable_permissionless_thaw: bool
    - Whether or not to enable the permissionless thaw for a given token

- enable_permissionless_freeze: bool
    - Whether or not to enable the permissionless freeze for a given token

### Instructions

- set_authority
    - Allows changing the given authority on a MintConfig account

- create_config
    - Creates a MintConfig account

- Can only be called once per Mint
    - Optionally safely sets the freeze authority in the given mint (needs freeze authority to call as signer)

- set_gating_program
    - Changes the MintConfig.gating_program.

- forfeit_freeze_authority
    - Transfers the mint freeze authority back to the freeze authority

- thaw (permissioned)
    - Given that the program holds the freeze authority, it needs to implement a regular permissioned thaw. Only callable by MintConfig.authority.

- freeze (permissioned)
    - Given that the program holds the freeze authority, it needs to implement a regular permissioned freeze. Only callable by MintConfig.authority.

- thaw_permissionless
    - Calls the gating instruction to decide whether or not the caller should be able to thaw a token account permissionless

- freeze_permissionless
    - Calls the gating instruction to decide whether or not the caller should be able to freeze a token account permissionless

- thaw_permissionless_idempotent
    - Idempotent version of thaw_permissionless. Will return success early if token account state is set to `Initialized`.

- freeze_permissionless_idempotent
    - Idempotent version of freeze_permissionlesss. Will return success early if token account state is set to `Frozen`.

### Interface

The interface needs two methods, both with optional implementations (should return an error when not implemented). Each implemented instruction requires the respective extra account metas PDA created and populated in order to enable account dependency resolution:

- Permissionless thaw
    - Discriminator_hash_input: “efficient-allow-block-list-standard:can-thaw-permissionless”
    - Discriminator: [u8; 8] = [8, 175, 169, 129, 137, 74, 61, 241]
    - Extra Accounts Metas seeds: [b”thaw-extra-account-metas”, mint_address]
    - Remaining instruction data: [  ]
    - Accounts: [caller, token account, mint, token account owner, flag account, extra-account-metas]
    - Remaining accounts: accounts as defined in extra account metas PDA

- Permissionless freeze
    - Discriminator_hash_input: “efficient-allow-block-list-standard:can-freeze-permissionless"
    - Discriminator: [u8; 8] = [214, 141, 109, 75, 248, 1, 45, 29]
    - Extra Account Metas seeds: [b”freeze-extra-account-metas”, mint_address]
    - Remaining instruction data: [  ]
    - Accounts: [caller, token account, mint, token account owner, flag account, extra-account-metas]
    - Remaining accounts: accounts as defined in extra account metas PDA


In order for gate programs to have assurances as to whether they're being called under the right circunstances, a Flag Account is created for the duration of the `can-thaw-permissionless` and `can-freeze-permissionless` operations. This account is created with 0 lamports and a single byte of data which is set to 1. Programs that require some level of bookkeeping should check that the flag account fulfills 2 constraints:
 - Is owned by the Token ACL program
 - Data: [u8; 1] = [1]

Extra accounts format: [github.com/solana-program/libraries/tree/main/tlv-account-resolution](http://github.com/solana-program/libraries/tree/main/tlv-account-resolution)

Unlike the transfer-hook interface, we’re not providing interface instructions to populate the extra account metas given that this is widely dependent on the protocol and user implementation.

## Gate Program

The Gate Program in this workflow is responsible for implementing the interface instructions. The instructions themselves will not call the T22 to freeze/thaw, but simply check whether the caller should freeze or thaw.

For each of the thaw and freeze instructions, the smart contract also needs to create and populate the respective extra metas account.

The instructions should return an error value when the given operation is not supported, not valid, or doesn’t pass all checks to occur in a permissionless manner.

Here are some common workflows and how to execute them:

**Permissionless thaw**

- This TA owner is blocked from interacting with my token?
    - Yes: Return failure
    - No: return success

- This operation is supported permissionlessly in my contract?
    - Yes: execute other checks
    - No: return failure

- This TA owner is allowed to interact with my token?
    - Yes: return success
    - No: return failure

**Permissionless freeze**

- This TA owner is blocked from interacting with my token?
    - Yes: Return success
    - No: return failure

- This operation is supported permissionlessly in my contract?
    - Yes: execute other checks
    - No: return failure

- This TA owner is allowed to interact with my token?
    - Yes: return failure
    - No: return success

## SDKs

### TypeScript

The typescript SDK should be able to:

- Detect whether a mint uses this standard or not
- Be able to craft a permissionless thaw instruction solely from the mint address
- Be able to craft a permissionless freeze instruction solely from the mint address
- Implement the methods to thaw and freeze permissioned
- Support the remaining methods to handle initialization and authority management
- Be aware that a token account may not exist during the extra metas account resolution steps

### Rust

The rust SDK should implement a similar functionality compared to the transfer-hook interface with an on-chain and an off-chain component.

The on-chain component serves to help the proxy program to parse the extra-account-metas and build the respective CPI into the user-defined program, while the off-chain component serves to help build transactions from off-chain rust programs.

Reference: [github.com/solana-program/transfer-hook/tree/main/interface](http://github.com/solana-program/transfer-hook/tree/main/interface)

## Workflow

Client workflow in order to detect if they should use custom logic to deal with permissionless thaw is as follow:
<img width="1862" height="2430" alt="mermaid-diagram-2025-06-18-193427" src="https://github.com/user-attachments/assets/5f8e91f3-09b5-4db8-b43d-b9ddf76f47c4" />

Execution workflow and transaction contents would be:
<img width="4220" height="2988" alt="mermaid-diagram-2025-06-17-231306" src="https://github.com/user-attachments/assets/45e88f4c-25d6-4ac4-9632-bc287fbc78f3" />

A protocol that creates a token account vault for a token that uses this standard wouldn't need to directly support the standard at the smart contract level. Upon creation of the token account, it can either call the permissionless thaw on the same transaction, or asynchronously using the CLI.

A protocol would need direct support if there are token transfers on the same instruction that creates the token account vault. Separating this into two distinct instructions (initialize + deposit) enables the workflow described in the previous paragraph instead.

# Security

The Token ACL solves the largest security concern in this system - the ability for a 3rd party to insert malicious instructions in unsuspecting users transactions. Standardizing a way for wallets/contracts/client software to introduce a new instruction to thaw token accounts right after creation is a sure way to enable bad actors.

The Token ACL solves this by de-escalating the permissions and acting as a proxy into the actual custom code that decides whether or not to act on the permissionless thaw and freeze operations.

# Implementations

Ongoing implementation: https://github.com/tiago18c/token-acl
Example Allow / Block list program that implements permissionless thaw: https://github.com/tiago18c/abl-srfc37