import {
  type Address,
  type Instruction,
  type TransactionSigner,
  AccountRole,
  type AccountMeta,
  type MaybeEncodedAccount,
  SolanaRpcApi,
  Rpc,
  MaybeAccount,
  address,
  lamports,
  fetchEncodedAccount,
} from '@solana/kit';
import { findMintConfigPda } from './generated/pdas/mintConfig';
import {
  findFreezeExtraMetasAccountPda,
  findThawExtraMetasAccountPda,
  getFreezePermissionlessInstruction,
  getMintConfigDecoder,
  getThawPermissionlessInstruction,
  findFlagAccountPda,
  getThawPermissionlessIdempotentInstruction,
  getFreezePermissionlessIdempotentInstruction,
  TOKEN_ACL_PROGRAM_ADDRESS,
  MintConfig,
  decodeMintConfig,
} from './generated';
import { resolveExtraMetas } from './tlv-account-resolution/state';

import {
  AccountState,
  getCreateAssociatedTokenIdempotentInstruction,
  findAssociatedTokenPda,
  getMintDecoder,
  getTokenEncoder,
  Mint,
  TOKEN_2022_PROGRAM_ADDRESS,
} from '@solana-program/token-2022';

/**
 * Creates an instruction to permissionlessly thaw a token account including all extra meta account dependencies.
 * @param authority The caller of the instruction.
 * @param tokenAccount The token account to thaw.
 * @param mint The mint of the token account.
 * @param tokenAccountOwner The owner of the token account.
 * @param programAddress The address of the program.
 * @param accountRetriever A function to retrieve the account data for a given address.
 *  If the token account is being created in the same transaction, the function should mock the expected account data.
 * @returns The instruction to thaw the token account.
 */
export async function createThawPermissionlessInstructionWithExtraMetas(
  authority: TransactionSigner,
  tokenAccount: Address,
  mint: Address,
  tokenAccountOwner: Address,
  programAddress: Address,
  accountRetriever: (address: Address) => Promise<MaybeEncodedAccount<string>>
): Promise<Instruction> {
  const mintConfigPda = await findMintConfigPda({ mint }, { programAddress });
  const mintConfigAccount = await accountRetriever(mintConfigPda[0]);
  if (!mintConfigAccount.exists) {
    throw new Error('Mint config account not found');
  }
  const mintConfigData = getMintConfigDecoder().decode(mintConfigAccount.data);
  const flagAccount = await findFlagAccountPda(
    { tokenAccount },
    { programAddress }
  );

  const thawExtraMetas = await findThawExtraMetasAccountPda(
    { mint },
    { programAddress: mintConfigData.gatingProgram }
  );

  console.log(mintConfigData);
  console.log(thawExtraMetas[0]);

  const canThawPermissionlessInstruction =
    getCanThawOrFreezePermissionlessAccountMetas(
      authority.address,
      tokenAccount,
      mint,
      tokenAccountOwner,
      flagAccount[0],
      thawExtraMetas[0]
    );

  const thawAccountInstruction = getThawPermissionlessInstruction(
    {
      authority,
      tokenAccount,
      flagAccount: flagAccount[0],
      mint,
      mintConfig: mintConfigPda[0],
      tokenAccountOwner,
      gatingProgram: mintConfigData.gatingProgram,
    },
    {
      programAddress,
    }
  );

  const metas = await resolveExtraMetas(
    accountRetriever,
    thawExtraMetas[0],
    canThawPermissionlessInstruction,
    Buffer.from(thawAccountInstruction.data),
    mintConfigData.gatingProgram
  );

  const ix = {
    ...thawAccountInstruction,
    accounts: [...thawAccountInstruction.accounts!, ...metas.slice(5)],
  };
  return ix;
}

/**
 * Creates an instruction to permissionlessly thaw a token account including all extra meta account dependencies. This instruction is idempotent.
 * @param authority The caller of the instruction.
 * @param tokenAccount The token account to thaw.
 * @param mint The mint of the token account.
 * @param tokenAccountOwner The owner of the token account.
 * @param programAddress The address of the program.
 * @param accountRetriever A function to retrieve the account data for a given address.
 *  If the token account is being created in the same transaction, the function should mock the expected account data.
 * @returns The instruction to thaw the token account.
 */
export async function createThawPermissionlessIdempotentInstructionWithExtraMetas(
  authority: TransactionSigner,
  tokenAccount: Address,
  mint: Address,
  tokenAccountOwner: Address,
  programAddress: Address,
  accountRetriever: (address: Address) => Promise<MaybeEncodedAccount<string>>
): Promise<Instruction> {
  const mintConfigPda = await findMintConfigPda({ mint }, { programAddress });
  const mintConfigAccount = await accountRetriever(mintConfigPda[0]);
  if (!mintConfigAccount.exists) {
    throw new Error('Mint config account not found');
  }
  const mintConfigData = getMintConfigDecoder().decode(mintConfigAccount.data);
  const flagAccount = await findFlagAccountPda(
    { tokenAccount },
    { programAddress }
  );

  const thawExtraMetas = await findThawExtraMetasAccountPda(
    { mint },
    { programAddress: mintConfigData.gatingProgram }
  );

  console.log(mintConfigData);
  console.log(thawExtraMetas[0]);

  const canThawPermissionlessInstruction =
    getCanThawOrFreezePermissionlessAccountMetas(
      authority.address,
      tokenAccount,
      mint,
      tokenAccountOwner,
      flagAccount[0],
      thawExtraMetas[0]
    );

  const thawAccountInstruction = getThawPermissionlessIdempotentInstruction(
    {
      authority,
      tokenAccount,
      flagAccount: flagAccount[0],
      mint,
      mintConfig: mintConfigPda[0],
      tokenAccountOwner,
      gatingProgram: mintConfigData.gatingProgram,
    },
    {
      programAddress,
    }
  );

  const metas = await resolveExtraMetas(
    accountRetriever,
    thawExtraMetas[0],
    canThawPermissionlessInstruction,
    Buffer.from(thawAccountInstruction.data),
    mintConfigData.gatingProgram
  );

  const ix = {
    ...thawAccountInstruction,
    accounts: [...thawAccountInstruction.accounts!, ...metas.slice(5)],
  };
  return ix;
}

function getCanThawOrFreezePermissionlessAccountMetas(
  authority: Address,
  tokenAccount: Address,
  mint: Address,
  owner: Address,
  flagAccount: Address,
  extraMetas: Address
): AccountMeta[] {
  return [
    { address: authority, role: AccountRole.READONLY },
    { address: tokenAccount, role: AccountRole.READONLY },
    { address: mint, role: AccountRole.READONLY },
    { address: owner, role: AccountRole.READONLY },
    { address: flagAccount, role: AccountRole.READONLY },
    { address: extraMetas, role: AccountRole.READONLY },
  ];
}

/**
 * Creates an instruction to permissionlessly freeze a token account including all extra meta account dependencies.
 * @param authority The caller of the instruction.
 * @param tokenAccount The token account to freeze.
 * @param mint The mint of the token account.
 * @param tokenAccountOwner The owner of the token account.
 * @param programAddress The address of the program.
 * @param accountRetriever A function to retrieve the account data for a given address.
 *  If the token account is being created in the same transaction, the function should mock the expected account data.
 * @returns The instruction to freeze the token account.
 */
export async function createFreezePermissionlessInstructionWithExtraMetas(
  authority: TransactionSigner,
  tokenAccount: Address,
  mint: Address,
  tokenAccountOwner: Address,
  programAddress: Address,
  accountRetriever: (address: Address) => Promise<MaybeEncodedAccount<string>>
): Promise<Instruction> {
  const mintConfigPda = await findMintConfigPda({ mint });
  const mintConfigAccount = await accountRetriever(mintConfigPda[0]);
  if (!mintConfigAccount.exists) {
    throw new Error('Mint config account not found');
  }
  const mintConfigData = getMintConfigDecoder().decode(mintConfigAccount.data);
  const flagAccount = await findFlagAccountPda(
    { tokenAccount },
    { programAddress }
  );

  const freezeExtraMetas = await findFreezeExtraMetasAccountPda(
    { mint },
    { programAddress: mintConfigData.gatingProgram }
  );

  const freezeAccountInstruction = getFreezePermissionlessInstruction({
    authority,
    tokenAccount,
    mint,
    flagAccount: flagAccount[0],
    mintConfig: mintConfigPda[0],
    tokenAccountOwner,
    gatingProgram: mintConfigData.gatingProgram,
  });

  const canFreezePermissionlessInstruction =
    getCanThawOrFreezePermissionlessAccountMetas(
      authority.address,
      tokenAccount,
      mint,
      tokenAccountOwner,
      flagAccount[0],
      freezeExtraMetas[0]
    );

  const metas = await resolveExtraMetas(
    accountRetriever,
    freezeExtraMetas[0],
    canFreezePermissionlessInstruction,
    Buffer.from(freezeAccountInstruction.data),
    mintConfigData.gatingProgram
  );

  const ix = {
    ...freezeAccountInstruction,
    accounts: [...freezeAccountInstruction.accounts!, ...metas.slice(5)],
  };
  return ix;
}

/**
 * Creates an instruction to permissionlessly freeze a token account including all extra meta account dependencies. This instruction is idempotent.
 * @param authority The caller of the instruction.
 * @param tokenAccount The token account to freeze.
 * @param mint The mint of the token account.
 * @param tokenAccountOwner The owner of the token account.
 * @param programAddress The address of the program.
 * @param accountRetriever A function to retrieve the account data for a given address.
 *  If the token account is being created in the same transaction, the function should mock the expected account data.
 * @returns The instruction to freeze the token account.
 */
export async function createFreezePermissionlessIdempotentInstructionWithExtraMetas(
  authority: TransactionSigner,
  tokenAccount: Address,
  mint: Address,
  tokenAccountOwner: Address,
  programAddress: Address,
  accountRetriever: (address: Address) => Promise<MaybeEncodedAccount<string>>
): Promise<Instruction> {
  const mintConfigPda = await findMintConfigPda({ mint });
  const mintConfigAccount = await accountRetriever(mintConfigPda[0]);
  if (!mintConfigAccount.exists) {
    throw new Error('Mint config account not found');
  }
  const mintConfigData = getMintConfigDecoder().decode(mintConfigAccount.data);
  const flagAccount = await findFlagAccountPda(
    { tokenAccount },
    { programAddress }
  );

  const freezeExtraMetas = await findFreezeExtraMetasAccountPda(
    { mint },
    { programAddress: mintConfigData.gatingProgram }
  );

  const freezeAccountInstruction = getFreezePermissionlessIdempotentInstruction(
    {
      authority,
      tokenAccount,
      mint,
      flagAccount: flagAccount[0],
      mintConfig: mintConfigPda[0],
      tokenAccountOwner,
      gatingProgram: mintConfigData.gatingProgram,
    }
  );

  const canFreezePermissionlessInstruction =
    getCanThawOrFreezePermissionlessAccountMetas(
      authority.address,
      tokenAccount,
      mint,
      tokenAccountOwner,
      flagAccount[0],
      freezeExtraMetas[0]
    );

  const metas = await resolveExtraMetas(
    accountRetriever,
    freezeExtraMetas[0],
    canFreezePermissionlessInstruction,
    Buffer.from(freezeAccountInstruction.data),
    mintConfigData.gatingProgram
  );

  const ix = {
    ...freezeAccountInstruction,
    accounts: [...freezeAccountInstruction.accounts!, ...metas.slice(5)],
  };
  return ix;
}

/**
 * Validates that a mint is a valid Token ACL mint.
 * This method does extensive checks to ensure everything is properly configured.
 * @param rpc The RPC client.
 * @param mint The mint to validate.
 * @returns True if the mint is a valid Token ACL mint, false otherwise.
 */
export async function isValidTokenAclMint(
  rpc: Rpc<SolanaRpcApi>,
  mint: Address
) {
  const mintAccount = await rpc
    .getAccountInfo(mint, { commitment: 'confirmed', encoding: 'base64' })
    .send();
  if (mintAccount.value?.owner != TOKEN_2022_PROGRAM_ADDRESS) {
    return false;
  }
  const mintData = getMintDecoder().decode(
    new Uint8Array(Buffer.from(mintAccount.value.data[0], 'base64'))
  );

  if (mintData.freezeAuthority.__option === 'None') {
    return false;
  }

  const freezeAuthority = await rpc
    .getAccountInfo(mintData.freezeAuthority.value, {
      commitment: 'confirmed',
      encoding: 'base64',
    })
    .send();

  if (freezeAuthority.value?.owner != TOKEN_ACL_PROGRAM_ADDRESS) {
    return false;
  }

  const mintConfig = await getTokenAclMintConfig(rpc, mint);
  if (!mintConfig.exists) {
    return false;
  }

  const gateProgram = await getTokenAclGateProgramFromMint(mintData);
  if (!gateProgram) {
    return false;
  }
  if (gateProgram != mintConfig.data.gatingProgram) {
    return false;
  }

  return true;
}

/**
 * Checks if a mint is a Token ACL mint using the mint's metadata.
 * @param rpc The RPC client.
 * @param mint The mint to check.
 * @returns True if the mint is a Token ACL mint, false otherwise.
 */
export async function isTokenAclMint(
  rpc: Rpc<SolanaRpcApi>,
  mint: Address
): Promise<boolean> {
  const gateProgram = await getTokenAclGateProgram(rpc, mint);
  if (!gateProgram) {
    return false;
  }
  return true;
}

/**
 * Checks if a mint is a Token ACL mint using the mint's metadata.
 * @param mint The mint to check.
 * @returns True if the mint is a Token ACL mint, false otherwise.
 */
export async function isTokenAclMintFromMint(mint: Mint): Promise<boolean> {
  const gateProgram = await getTokenAclGateProgramFromMint(mint);
  if (!gateProgram) {
    return false;
  }
  return true;
}

/**
 * Gets the Token ACL gate program from a mint's metadata.
 * @param mint The mint to get the gate program from.
 * @returns The Token ACL gate program, or undefined if the mint is not a Token ACL mint.
 */
export function getTokenAclGateProgramFromMint(
  mint: Mint
): Address | undefined {
  if (mint.extensions.__option === 'None') {
    return undefined;
  }
  const extensions = mint.extensions.value;
  const metadataExtension = extensions.find(
    (extension) => extension.__kind == 'TokenMetadata'
  );

  if (!metadataExtension) {
    return undefined;
  }
  const gateProgram = metadataExtension.additionalMetadata.get('token_acl');
  if (!gateProgram) {
    return undefined;
  }
  return address(gateProgram);
}

/**
 * Gets the Token ACL gate program from a mint's metadata.
 * @param rpc The RPC client.
 * @param mint The mint to get the gate program from.
 * @returns The Token ACL gate program, or undefined if the mint is not a Token ACL mint.
 */
export async function getTokenAclGateProgram(
  rpc: Rpc<SolanaRpcApi>,
  mint: Address
): Promise<Address | undefined> {
  const mintAccount = await rpc
    .getAccountInfo(mint, { commitment: 'confirmed', encoding: 'base64' })
    .send();
  if (mintAccount.value?.owner != TOKEN_2022_PROGRAM_ADDRESS) {
    return undefined;
  }
  const mintData = getMintDecoder().decode(
    new Uint8Array(Buffer.from(mintAccount.value.data[0], 'base64'))
  );

  return getTokenAclGateProgramFromMint(mintData);
}

/**
 * Gets the Token ACL mint config for a given a mint.
 * @param rpc The RPC client.
 * @param mint The mint to get the mint config from.
 * @returns The Token ACL mint config, or undefined if the mint is not a Token ACL mint.
 */
export async function getTokenAclMintConfig(
  rpc: Rpc<SolanaRpcApi>,
  mint: Address
): Promise<MaybeAccount<MintConfig, string>> {
  const mintAccount = await fetchEncodedAccount(rpc, mint, {
    commitment: 'confirmed',
  });
  if (!mintAccount.exists) {
    return {
      exists: false,
      address: address('11111111111111111111111111111111'),
    };
  }
  const mintData = getMintDecoder().decode(mintAccount.data);
  if (mintData.freezeAuthority.__option === 'None') {
    return {
      exists: false,
      address: address('11111111111111111111111111111111'),
    };
  }
  const freezeAuthority = await fetchEncodedAccount(
    rpc,
    mintData.freezeAuthority.value,
    { commitment: 'confirmed' }
  );
  if (
    !freezeAuthority.exists ||
    freezeAuthority.programAddress != TOKEN_ACL_PROGRAM_ADDRESS
  ) {
    return {
      exists: false,
      address: address('11111111111111111111111111111111'),
    };
  }
  return decodeMintConfig(freezeAuthority as MaybeEncodedAccount<string>);
}

/**
 * Checks if a mint uses permissionless thaw.
 * @param rpc The RPC client.
 * @param mint The mint to check.
 * @returns True if the mint uses permissionless thaw, false otherwise.
 */
export async function usesPermissionlessThaw(
  rpc: Rpc<SolanaRpcApi>,
  mint: Address
): Promise<boolean> {
  const mintConfig = await getTokenAclMintConfig(rpc, mint);
  if (!mintConfig.exists) {
    return false;
  }
  return mintConfig.data.enablePermissionlessThaw;
}

/**
 * Checks if a mint uses permissionless freeze.
 * @param rpc The RPC client.
 * @param mint The mint to check.
 * @returns True if the mint uses permissionless freeze, false otherwise.
 */
export async function usesPermissionlessFreeze(
  rpc: Rpc<SolanaRpcApi>,
  mint: Address
): Promise<boolean> {
  const mintConfig = await getTokenAclMintConfig(rpc, mint);
  if (!mintConfig.exists) {
    return false;
  }
  return mintConfig.data.enablePermissionlessFreeze;
}

/**
 * Builds the instructions to create a token account and to thaw it permissionlessly.
 * This method
 * @param rpc The RPC client.
 * @param mint The mint to create the token account for.
 * @param mintAddress The mint address.
 * @param tokenAccountOwner The owner of the token account.
 * @param payer The payer of the transaction.
 * @returns The instructions to create the token account.
 */
export async function createTokenAccountWithAcl(
  rpc: Rpc<SolanaRpcApi>,
  mint: Mint,
  mintAddress: Address,
  tokenAccountOwner: Address,
  payer: TransactionSigner
): Promise<Instruction[]> {
  // Derive ATA for wallet address
  const [tokenAccountAddress] = await findAssociatedTokenPda({
    mint: mintAddress,
    owner: tokenAccountOwner,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });

  const createAssociatedTokenAccountInstruction =
    getCreateAssociatedTokenIdempotentInstruction({
      owner: tokenAccountOwner,
      mint: mintAddress,
      ata: tokenAccountAddress,
      payer: payer,
      tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
    });

  const thawInstruction = await createThawPermissionlessInstructionFromMint(
    rpc,
    mint,
    mintAddress,
    tokenAccountOwner,
    tokenAccountAddress,
    payer,
    true
  );

  return [createAssociatedTokenAccountInstruction, thawInstruction];
}

/**
 * Builds the instruction to thaw a token account permissionlessly from a token-acl mint.
 * This method does not create the token account.
 * @param rpc The RPC client.
 * @param mint The mint to create the token account for.
 * @param mintAddress The mint address.
 * @param tokenAccountOwner The owner of the token account.
 * @param tokenAccountAddress The address of the token account.
 * @param payer The payer of the transaction.
 * @param idempotent Whether to use idempotent instruction variant.
 * @returns The instructions to create the token account.
 */
export async function createThawPermissionlessInstructionFromMint(
  rpc: Rpc<SolanaRpcApi>,
  mint: Mint,
  mintAddress: Address,
  tokenAccountOwner: Address,
  tokenAccountAddress: Address,
  payer: TransactionSigner,
  idempotent: boolean = false
): Promise<Instruction> {
  if (mint.extensions.__option === 'None') {
    throw new Error('Mint is not a valid token acl mint');
  }
  const gateProgramAddress = getTokenAclGateProgramFromMint(mint);
  if (!gateProgramAddress) {
    throw new Error('Mint is not a valid token mint');
  }
  const flagAccount = await findFlagAccountPda(
    { tokenAccount: tokenAccountAddress },
    { programAddress: TOKEN_ACL_PROGRAM_ADDRESS }
  );
  const thawExtraMetas = await findThawExtraMetasAccountPda(
    { mint: mintAddress },
    { programAddress: gateProgramAddress }
  );
  const mintConfig = await findMintConfigPda(
    { mint: mintAddress },
    { programAddress: TOKEN_ACL_PROGRAM_ADDRESS }
  );

  const canThawPermissionlessInstruction =
    getCanThawOrFreezePermissionlessAccountMetas(
      payer.address,
      tokenAccountAddress,
      mintAddress,
      tokenAccountOwner,
      flagAccount[0],
      thawExtraMetas[0]
    );

  const thawAccountInstruction = idempotent
    ? getThawPermissionlessIdempotentInstruction(
        {
          authority: payer,
          tokenAccount: tokenAccountAddress,
          flagAccount: flagAccount[0],
          mint: mintAddress,
          mintConfig: mintConfig[0],
          tokenAccountOwner,
          gatingProgram: gateProgramAddress,
        },
        {
          programAddress: TOKEN_ACL_PROGRAM_ADDRESS,
        }
      )
    : getThawPermissionlessInstruction(
        {
          authority: payer,
          tokenAccount: tokenAccountAddress,
          flagAccount: flagAccount[0],
          mint: mintAddress,
          mintConfig: mintConfig[0],
          tokenAccountOwner,
          gatingProgram: gateProgramAddress,
        },
        {
          programAddress: TOKEN_ACL_PROGRAM_ADDRESS,
        }
      );

  const accountRetriever = async (address: Address) => {
    if (address === tokenAccountAddress) {
      const data = getTokenEncoder().encode({
        amount: 0,
        closeAuthority: null,
        delegate: null,
        delegatedAmount: 0,
        extensions: null,
        isNative: null,
        mint: mintAddress,
        owner: tokenAccountOwner,
        state: AccountState.Frozen,
      });
      return {
        exists: true,
        address,
        data: new Uint8Array(data),
        executable: false,
        lamports: lamports(BigInt(2157600)),
        programAddress: TOKEN_2022_PROGRAM_ADDRESS,
        space: BigInt(data.byteLength),
      };
    }
    return await fetchEncodedAccount(rpc, address);
  };

  const metas = await resolveExtraMetas(
    accountRetriever,
    thawExtraMetas[0],
    canThawPermissionlessInstruction,
    Buffer.from(thawAccountInstruction.data),
    gateProgramAddress
  );

  const ix: Instruction = {
    ...thawAccountInstruction,
    accounts: [...thawAccountInstruction.accounts!, ...metas.slice(5)],
  };

  return ix;
}
