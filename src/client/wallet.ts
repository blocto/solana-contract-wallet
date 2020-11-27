/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/ban-ts-comment */

// @ts-ignore
import * as BufferLayout from 'buffer-layout';

import {
  PublicKey,
  TransactionInstruction,
} from '@solana/web3.js';

export class Wallet {
  static createHelloTransaction(
    programId: PublicKey,
    dest: PublicKey,
  ): TransactionInstruction {
    const dataLayout = BufferLayout.struct([
      BufferLayout.u8('instruction')
    ]);

    const data = Buffer.alloc(dataLayout.span);
    dataLayout.encode(
      {
        instruction: 3, // MintTo instruction
      },
      data,
    );

    return new TransactionInstruction({
      keys: [{pubkey: dest, isSigner: false, isWritable: true}],
      programId: programId,
      data,
    });
  }
}
