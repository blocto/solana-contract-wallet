/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/ban-ts-comment */

// @ts-ignore
import * as BufferLayout from 'buffer-layout';

import {Account, PublicKey, TransactionInstruction} from '@solana/web3.js';

export enum Instruction {
  AddOwner = 0,
  RemoveOwner,
  Recovery,
  Invoke,
  Revoke,
  Hello,
}

const AccountMetaLayout = BufferLayout.struct([
  BufferLayout.seq(BufferLayout.u8(), 32, 'pubkey'),
  BufferLayout.u8('isSigner'),
  BufferLayout.u8('isWritable'),
]);

export class Wallet {
  static encodeInstruction(instruction: TransactionInstruction): Buffer {
    const dataLayout = BufferLayout.struct([
      BufferLayout.seq(BufferLayout.u8(), 32, 'programId'),
      BufferLayout.u16('keysLength'),
      BufferLayout.seq(AccountMetaLayout, instruction.keys.length, 'keys'),
      BufferLayout.seq(BufferLayout.u8(), instruction.data.length, 'data'),
    ]);

    const data = Buffer.alloc(dataLayout.span);
    dataLayout.encode(
      {
        programId: instruction.programId.toBuffer(),
        keysLength: instruction.keys.length,
        keys: instruction.keys.map(key => ({
          pubkey: key.pubkey.toBuffer(),
          isSigner: key.isSigner,
          isWritable: key.isWritable,
        })),
        data: instruction.data,
      },
      data,
    );

    return data;
  }

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
      isWritable: false,
    }));

    keys = [
      {
        pubkey: walletPubkey,
        isSigner: false,
        isWritable: true,
      },
      ...keys,
    ];

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
      isWritable: false,
    }));

    keys = [
      {
        pubkey: walletPubkey,
        isSigner: false,
        isWritable: true,
      },
      ...keys,
    ];

    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }

  static async createInvokeTransaction(
    programId: PublicKey,
    walletPubkey: PublicKey,
    internalInstruction: TransactionInstruction,
    signers: Array<Account>,
  ): Promise<TransactionInstruction> {
    const internalInstructionData = Wallet.encodeInstruction(
      internalInstruction,
    );

    const dataLayout = BufferLayout.struct([
      BufferLayout.u8('instruction'),
      BufferLayout.seq(
        BufferLayout.u8(),
        internalInstructionData.length,
        'data',
      ),
    ]);

    const data = Buffer.alloc(dataLayout.span);
    dataLayout.encode(
      {
        instruction: Instruction.Invoke,
        data: internalInstructionData,
      },
      data,
    );

    let keys = signers.map(signer => ({
      pubkey: signer.publicKey,
      isSigner: true,
      isWritable: false,
    }));

    const derivedPubkey = await PublicKey.createProgramAddress(
      [walletPubkey.toBuffer()],
      programId,
    );

    keys = [
      // wallet account used
      {
        pubkey: walletPubkey,
        isSigner: false,
        isWritable: true,
      },
      // cooresponding program account
      {
        pubkey: derivedPubkey,
        isSigner: false,
        isWritable: true,
      },
      // target instruction program
      {
        pubkey: internalInstruction.programId,
        isSigner: false,
        isWritable: false,
      },
      ...internalInstruction.keys.filter(
        key => key.pubkey.toBase58() !== derivedPubkey.toBase58(),
      ),
      ...keys,
    ];

    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }

  static createHelloTransaction(
    programId: PublicKey,
    walletPubkey: PublicKey,
    signers: Array<Account>,
  ): TransactionInstruction {
    const dataLayout = BufferLayout.struct([BufferLayout.u8('instruction')]);

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
      isWritable: false,
    }));

    return new TransactionInstruction({
      keys: [
        {pubkey: walletPubkey, isSigner: false, isWritable: true},
        ...keys,
      ],
      programId,
      data,
    });
  }
}
