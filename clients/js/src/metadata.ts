import { getUpdateTokenMetadataFieldInstruction, TOKEN_2022_PROGRAM_ADDRESS } from "@solana-program/token-2022";
import { Address, Instruction, TransactionSigner } from "@solana/kit";

export const TOKEN_ACL_METADATA_KEY = 'token_acl';

export function setTokenAclMetadata(
  metadataAuthority: TransactionSigner,
  mint: Address,
  gateProgram: Address,
) : Instruction {

  return getUpdateTokenMetadataFieldInstruction({
    field: {__kind: 'Key', fields: [TOKEN_ACL_METADATA_KEY]},
    value: gateProgram,
    metadata: mint,
    updateAuthority: metadataAuthority,
  }, {
    programAddress: TOKEN_2022_PROGRAM_ADDRESS,
  })
}