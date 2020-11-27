/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/ban-ts-comment */

// @ts-ignore
import * as BufferLayout from 'buffer-layout';

import {
  Account,
  PublicKey,
  TransactionInstruction,
} from '@solana/web3.js';

export enum Instruction {
  AddOwner = 0,
  RemoveOwner,
  Invoke,
  Hello,
}

export class Wallet {
  static createAddOwnerTransaction(
    programId: PublicKey,
    walletPubkey: PublicKey,
    pubkey: PublicKey,
    weight: number,
    signers: Array<Account>,
  ): TransactionInstruction {
    const dataLayout = BufferLayout.struct([
      BufferLayout.u8('instruction'),
      BufferLayout.seq(BufferLayout.u8(), 32, 'pubkey'),
      BufferLayout.u16('weight'),
    ]);

    const data = Buffer.alloc(dataLayout.span);
    dataLayout.encode(
      {
        instruction: Instruction.AddOwner,
        pubkey: pubkey.toBuffer(),
        weight,
      },
      data,
    );

    let keys = signers.map(signer => ({
      pubkey: signer.publicKey,
      isSigner: true,
      isWritable: true,
    }))

    keys = [
      {
        pubkey: walletPubkey,
        isSigner: false,
        isWritable: true,
      },
      ...keys,
    ]

    return new TransactionInstruction({
      keys,
      programId: programId,
      data,
    });
  }

  static createRemoveOwnerTransaction(
    programId: PublicKey,
    walletPubkey: PublicKey,
    pubkey: PublicKey,
    signers: Array<Account>,
  ): TransactionInstruction {
    const dataLayout = BufferLayout.struct([
      BufferLayout.u8('instruction'),
      BufferLayout.seq(BufferLayout.u8(), 32, 'pubkey'),
      BufferLayout.u16('weight'),
    ]);

    const data = Buffer.alloc(dataLayout.span);
    dataLayout.encode(
      {
        instruction: Instruction.RemoveOwner,
        pubkey: pubkey.toBuffer(),
      },
      data,
    );

    let keys = signers.map(signer => ({
      pubkey: signer.publicKey,
      isSigner: true,
      isWritable: true,
    }))

    keys = [
      {
        pubkey: walletPubkey,
        isSigner: false,
        isWritable: true,
      },
      ...keys,
    ]

    return new TransactionInstruction({
      keys,
      programId: programId,
      data,
    });
  }

  static createHelloTransaction(
    programId: PublicKey,
    dest: PublicKey,
    signers: Array<Account>,
  ): TransactionInstruction {
    const dataLayout = BufferLayout.struct([
      BufferLayout.u8('instruction')
    ]);

    const data = Buffer.alloc(dataLayout.span);
    dataLayout.encode(
      {
        instruction: Instruction.Hello,
      },
      data,
    );

    const keys = signers.map(signer => ({
      pubkey: signer.publicKey,
      isSigner: true,
      isWritable: true,
    }))

    return new TransactionInstruction({
      keys: [{pubkey: dest, isSigner: false, isWritable: true}, ...keys],
      programId: programId,
      data,
    });
  }
}
