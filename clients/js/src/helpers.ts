import {
  type Address,
  type Instruction,
  type TransactionSigner,
  AccountRole,
  type AccountMeta,
  type MaybeEncodedAccount,
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
} from './generated';
import { resolveExtraMetas } from './tlv-account-resolution/state';

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
  const flagAccount = await findFlagAccountPda({ tokenAccount }, { programAddress });

  const thawExtraMetas = await findThawExtraMetasAccountPda(
    { mint },
    { programAddress: mintConfigData.gatingProgram }
  );

  console.log(mintConfigData);
  console.log(thawExtraMetas[0]);

  const canThawPermissionlessInstruction = getCanThawPermissionlessAccountMetas(
    authority.address,
    tokenAccount,
    mint,
    tokenAccountOwner,
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
    accounts: [...thawAccountInstruction.accounts!, ...metas.slice(4)],
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
  const flagAccount = await findFlagAccountPda({ tokenAccount }, { programAddress });

  const thawExtraMetas = await findThawExtraMetasAccountPda(
    { mint },
    { programAddress: mintConfigData.gatingProgram }
  );

  console.log(mintConfigData);
  console.log(thawExtraMetas[0]);

  const canThawPermissionlessInstruction = getCanThawPermissionlessAccountMetas(
    authority.address,
    tokenAccount,
    mint,
    tokenAccountOwner,
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
    accounts: [...thawAccountInstruction.accounts!, ...metas.slice(4)],
  };
  return ix;
}

function getCanThawPermissionlessAccountMetas(
  authority: Address,
  tokenAccount: Address,
  mint: Address,
  owner: Address,
  extraMetasThaw: Address
): AccountMeta[] {
  return [
    { address: authority, role: AccountRole.READONLY },
    { address: tokenAccount, role: AccountRole.READONLY },
    { address: mint, role: AccountRole.READONLY },
    { address: owner, role: AccountRole.READONLY },
    { address: extraMetasThaw, role: AccountRole.READONLY },
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
  const flagAccount = await findFlagAccountPda({ tokenAccount }, { programAddress });

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

  const metas = await resolveExtraMetas(
    accountRetriever,
    freezeExtraMetas[0],
    freezeAccountInstruction.accounts,
    Buffer.from(freezeAccountInstruction.data),
    mintConfigData.gatingProgram
  );

  const ix = {
    ...freezeAccountInstruction,
    accounts: metas,
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
  const flagAccount = await findFlagAccountPda({ tokenAccount }, { programAddress });

  const freezeExtraMetas = await findFreezeExtraMetasAccountPda(
    { mint },
    { programAddress: mintConfigData.gatingProgram }
  );

  const freezeAccountInstruction = getFreezePermissionlessIdempotentInstruction({
    authority,
    tokenAccount,
    mint,
    flagAccount: flagAccount[0],
    mintConfig: mintConfigPda[0],
    tokenAccountOwner,
    gatingProgram: mintConfigData.gatingProgram,
  });

  const metas = await resolveExtraMetas(
    accountRetriever,
    freezeExtraMetas[0],
    freezeAccountInstruction.accounts,
    Buffer.from(freezeAccountInstruction.data),
    mintConfigData.gatingProgram
  );

  const ix = {
    ...freezeAccountInstruction,
    accounts: metas,
  };
  return ix;
}