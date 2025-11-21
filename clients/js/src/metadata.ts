import { getUpdateTokenMetadataFieldInstruction, TOKEN_2022_PROGRAM_ADDRESS } from "@solana-program/token-2022";
import { Address, createNoopSigner, Instruction } from "@solana/kit";

export const TOKEN_ACL_METADATA_KEY = 'token_acl';

export function setTokenAclMetadata(
  mint: Address,
  gateProgram: Address,
  metadataAuthority: Address,
) : Instruction {

  return getUpdateTokenMetadataFieldInstruction({
    field: {__kind: 'Key', fields: [TOKEN_ACL_METADATA_KEY]},
    value: gateProgram,
    metadata: mint,
    updateAuthority: createNoopSigner(metadataAuthority),
  }, {
    programAddress: TOKEN_2022_PROGRAM_ADDRESS,
  })
}