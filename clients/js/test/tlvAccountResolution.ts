
import { expect, use } from 'chai';
import chaiAsPromised from 'chai-as-promised';
import { describe, it } from 'mocha';
import { AccountRole, Lamports, MaybeEncodedAccount, address, lamports, Address } from '@solana/kit';
import {
    getTokenEncoder,
    AccountState,
    TOKEN_2022_PROGRAM_ADDRESS,
  } from 'gill/programs';

import { getExtraAccountMetas, resolveExtraMetas } from '../src/tlv-account-resolution/state.js';

use(chaiAsPromised);

describe('TLV Account Resolution', () => {
    it('should resolve extra account metas', async () => {
        const accountData = "CK+pgYlKPfFKAAAAAgAAAADG9sfma0ofMHpHuYNwkGzUVgFtBOjxZDELu64Rn7VO9gAAAQEMd2FsbGV0X2VudHJ5AwYEASAgAAAAAAAAAAAAAAAAAAA=";

        const buffer = Uint8Array.from(atob(accountData), c => c.charCodeAt(0));

        const myExistingAccount: MaybeEncodedAccount<'AgUrgFf6oGWpMUqiBga7cVDzQ4msDpbYVWV3mCmHEjPD'> = {
            exists: true,
            address: address('AgUrgFf6oGWpMUqiBga7cVDzQ4msDpbYVWV3mCmHEjPD'),
            data: buffer,
            executable: false,
            lamports: 0n as Lamports,
            programAddress: address('11111111111111111111111111111111'),
            space: 0n,
        };


        const extraMetas = getExtraAccountMetas(myExistingAccount);

        console.log(extraMetas);

        let previousMetas = [
            {
                address: address('CJ1rWHtECY89gJ78ojJetu1LcNKrFgV8sdEhZJ52YSjA'),
                role: AccountRole.READONLY,
            },
            {
                address: address('Abdy4aBmeCB2MoBtPCL2oXLjWZTnFmDHSnJKbwKN7Zb3'),
                role: AccountRole.READONLY,
            },
            {
                address: address('A5hvcDAW8YjRS2xZy4RywexvsngKrng7uPgHL3Yr53MJ'),
                role: AccountRole.READONLY,
            },
            {
                address: address('CJ1rWHtECY89gJ78ojJetu1LcNKrFgV8sdEhZJ52YSjA'),
                role: AccountRole.READONLY,
            },
            {
                address: address('CJ1rWHtECY89gJ78ojJetu1LcNKrFgV8sdEhZJ52YSjA'),
                role: AccountRole.READONLY,
            },
            {
                address: address('AgUrgFf6oGWpMUqiBga7cVDzQ4msDpbYVWV3mCmHEjPD'),
                role: AccountRole.READONLY,
            },
        ];

        const resolvedMetas = await resolveExtraMetas(
            async (addressToFetch) => { 
                console.log(addressToFetch);
                if (addressToFetch === 'AgUrgFf6oGWpMUqiBga7cVDzQ4msDpbYVWV3mCmHEjPD') {
                    return myExistingAccount;
                }
                if (addressToFetch === "Abdy4aBmeCB2MoBtPCL2oXLjWZTnFmDHSnJKbwKN7Zb3") {
                    const data = getTokenEncoder().encode({
                      amount: 0,
                      closeAuthority: null,
                      delegate: null,
                      delegatedAmount: 0,
                      extensions: null,
                      isNative: null,
                      mint: address('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'),
                      owner: address('CJ1rWHtECY89gJ78ojJetu1LcNKrFgV8sdEhZJ52YSjA'),
                      state: AccountState.Frozen,
                    });
                    return {
                      exists: true,
                      address:addressToFetch,
                      data: new Uint8Array(data),
                      executable: false,
                      lamports: lamports(BigInt(2157600)),
                      programAddress: address('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'),
                      space: BigInt(data.byteLength),
                    };
                  }
                return {address: addressToFetch, exists: false}
            },
            address('AgUrgFf6oGWpMUqiBga7cVDzQ4msDpbYVWV3mCmHEjPD'),
            previousMetas,
            Buffer.from([]),
            address('GATEzzqxhJnsWF6vHRsgtixxSB8PaQdcqGEVTEHWiULz'));

        console.log(resolvedMetas);
    });
});