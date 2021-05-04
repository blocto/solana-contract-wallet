/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/ban-ts-comment */

import {
  Account,
  Connection,
  BpfLoader,
  BPF_LOADER_PROGRAM_ID,
  PublicKey,
  LAMPORTS_PER_SOL,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import fs from 'mz/fs';

// @ts-ignore
import BufferLayout from 'buffer-layout';

import {Wallet} from './wallet';
import {url, urlTls} from './util/url';
import {Store} from './util/store';
import {newAccountWithLamports} from './util/new-account-with-lamports';

/**
 * Connection to the network
 */
let connection: Connection;

/**
 * Connection to the network
 */
let payerAccount: Account;

/**
 * Hello world's program id
 */
let programId: PublicKey;

/**
 * The public key of the wallet account
 */
let walletPubkey: PublicKey;

/**
 * The public key of the wallet account
 */
let wallet2Pubkey: PublicKey;

/**
 * The address generated from derive
 */
let programAddress: PublicKey;

/**
 * The owners of the account
 */
let owners: Array<Account>;

const pathToProgram = 'dist/program/wallet.so';

/**
 * Layout of the greeted account data
 */
const walletAccountDataLayout = BufferLayout.struct([
  BufferLayout.u8('state'),
  BufferLayout.seq(
    BufferLayout.struct([
      BufferLayout.seq(BufferLayout.u8(), 32, 'pubkey'),
      BufferLayout.u16('weight'),
    ]),
    101,
    'owners',
  ),
]);

async function storeCurrentData(): Promise<void> {
  // Save this info for next time
  const store = new Store();
  await store.save('config.json', {
    url: urlTls,
    programId: programId.toBase58(),
    walletPubkey: walletPubkey.toBase58(),
    wallet2Pubkey: wallet2Pubkey.toBase58(),
    programAddress: programAddress.toBase58(),
    owners: JSON.stringify(
      owners.map(({secretKey}) => ({secretKey: secretKey.toString()})),
    ),
  });
}

async function loadCurrentData(): Promise<void> {
  // Save this info for next time
  const store = new Store();

  const config = await store.load('config.json');
  programId = new PublicKey(config.programId);
  walletPubkey = new PublicKey(config.walletPubkey);
  wallet2Pubkey = new PublicKey(config.wallet2Pubkey);
  programAddress = new PublicKey(config.programAddress);
  const ownersRaw = JSON.parse(config.owners);
  owners = ownersRaw.map(({secretKey}: any) => {
    const keyArray = secretKey
      .split(',')
      .map((number: string) => parseInt(number, 10));
    const account = new Account(keyArray);
    console.log('Loaded account', account.publicKey.toBase58());
    return account;
  });
  await connection.getAccountInfo(programId);
  console.log('Program already loaded to account', programId.toBase58());
}

/**
 * Establish a connection to the cluster
 */
export async function establishConnection(): Promise<void> {
  connection = new Connection(url, 'singleGossip');
  const version = await connection.getVersion();
  console.log('Connection to cluster established:', url, version);
}

/**
 * Establish an account to pay for everything
 */
export async function establishPayer(): Promise<void> {
  if (!payerAccount) {
    let fees = 0;
    const {feeCalculator} = await connection.getRecentBlockhash();

    // Calculate the cost to load the program
    const data = await fs.readFile(pathToProgram);
    const NUM_RETRIES = 500; // allow some number of retries
    fees +=
      feeCalculator.lamportsPerSignature *
        (BpfLoader.getMinNumSignatures(data.length) + NUM_RETRIES) +
      (await connection.getMinimumBalanceForRentExemption(data.length));

    // Calculate the cost to fund the greeter account
    fees +=
      2 *
      (await connection.getMinimumBalanceForRentExemption(
        walletAccountDataLayout.span,
      ));

    // Calculate the cost of sending the transactions
    fees += feeCalculator.lamportsPerSignature * 200; // wag

    // Fund a new payer via airdrop
    payerAccount = await newAccountWithLamports(connection, fees);
  }

  const lamports = await connection.getBalance(payerAccount.publicKey);

  console.log(
    'Using account',
    payerAccount.publicKey.toBase58(),
    'containing',
    lamports / LAMPORTS_PER_SOL,
    'Sol to process requests',
  );
}

/**
 * Load the hello world BPF program if not already loaded
 */
export async function loadProgram(): Promise<void> {
  // Check if the program has already been loaded
  try {
    await loadCurrentData();
    return;
  } catch (err) {
    // try to load the program
  }

  // Load the program
  console.log('Loading wallet program...');
  const data = await fs.readFile(pathToProgram);
  const programAccount = new Account();
  await BpfLoader.load(
    connection,
    payerAccount,
    programAccount,
    data,
    BPF_LOADER_PROGRAM_ID,
  );
  programId = programAccount.publicKey;
  console.log('Program loaded to account', programId.toBase58());

  // Do calculations
  const space = walletAccountDataLayout.span;
  // console.log('Account storage size', space)
  const lamports = await connection.getMinimumBalanceForRentExemption(
    walletAccountDataLayout.span,
  );

  // console.log('Account lamports', lamports)
  // Create the wallet account
  owners = [];
  let walletAccount = new Account();
  while (!programAddress) {
    try {
      walletAccount = new Account();
      walletPubkey = walletAccount.publicKey;
      programAddress = await PublicKey.createProgramAddress(
        [walletPubkey.toBuffer()],
        programId,
      );

      console.log(
        'Creating normal account',
        walletPubkey.toBase58(),
        'to store wallet data',
      );
      console.log('Derived address: ', programAddress.toBase58());

      const wallet2Account = new Account();
      wallet2Pubkey = wallet2Account.publicKey;
      console.log(
        'Creating contract controlled account',
        wallet2Pubkey.toBase58(),
        'to store wallet data',
      );

      const transaction = new Transaction()
        .add(
          SystemProgram.createAccount({
            fromPubkey: payerAccount.publicKey,
            newAccountPubkey: walletPubkey,
            lamports,
            space,
            programId,
          }),
        )
        .add(
          SystemProgram.createAccount({
            fromPubkey: payerAccount.publicKey,
            newAccountPubkey: wallet2Pubkey,
            lamports,
            space,
            programId,
          }),
        );

      await sendAndConfirmTransaction(
        connection,
        transaction,
        [payerAccount, walletAccount, wallet2Account],
        {
          commitment: 'singleGossip',
          preflightCommitment: 'singleGossip',
        },
      );
    } catch (err) {
      // retry wallet key
    }
  }

  console.log('Accounts created');

  // Initialize wallet 1
  await addWallet1Owner(1000);

  // Initialize wallet 2
  await addWallet2Owner(1000);
  await addWallet2Owner(500);
  await addContractWalletAsOwner();

  await storeCurrentData();
}

/**
 * Say hello
 */
export async function sayHello(): Promise<void> {
  const signers = owners.length ? [owners[0]] : [];

  console.log('Saying hello to', walletPubkey.toBase58());
  const instruction = Wallet.createHelloTransaction(
    programId,
    walletPubkey,
    signers,
  );
  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [payerAccount, ...signers],
    {
      commitment: 'singleGossip',
      preflightCommitment: 'singleGossip',
    },
  );
}

/**
 * Add new owner
 */
export async function addWallet1Owner(weight: number): Promise<void> {
  const signers = owners.length ? [owners[0]] : [];

  console.log('Adding a new owner to', walletPubkey.toBase58());
  const newOwnerAccount = new Account();
  owners = [...owners, newOwnerAccount];

  const instruction = Wallet.createAddOwnerTransaction(
    programId,
    walletPubkey,
    newOwnerAccount.publicKey,
    weight,
    signers,
  );

  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [payerAccount, ...signers],
    {
      commitment: 'singleGossip',
      preflightCommitment: 'singleGossip',
    },
  );

  await storeCurrentData();
}

export async function addWallet2Owner(weight: number): Promise<void> {
  const signers = owners.length > 1 ? [owners[1]] : [];

  console.log('Adding a new owner to', wallet2Pubkey.toBase58());
  const newOwnerAccount = new Account();
  owners = [...owners, newOwnerAccount];

  const instruction = Wallet.createAddOwnerTransaction(
    programId,
    wallet2Pubkey,
    newOwnerAccount.publicKey,
    weight,
    signers,
  );

  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [payerAccount, ...signers],
    {
      commitment: 'singleGossip',
      preflightCommitment: 'singleGossip',
    },
  );

  await storeCurrentData();
}

/**
 * Add new owner
 */
export async function addContractWalletAsOwner(): Promise<void> {
  const signers = owners.length > 1 ? [owners[1]] : [];

  console.log('Adding a new owner to', wallet2Pubkey.toBase58());

  const instruction = Wallet.createAddOwnerTransaction(
    programId,
    wallet2Pubkey,
    programAddress,
    500,
    signers,
  );

  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [payerAccount, ...signers],
    {
      commitment: 'singleGossip',
      preflightCommitment: 'singleGossip',
    },
  );

  await storeCurrentData();
}

/**
 * Remove owner
 */
export async function removeOwner(index: number): Promise<void> {
  const signers = owners.length ? [owners[0]] : [];

  console.log('Removing an owner from', walletPubkey.toBase58());

  const instruction = Wallet.createRemoveOwnerTransaction(
    programId,
    walletPubkey,
    owners[index].publicKey,
    signers,
  );

  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [payerAccount, ...signers],
    {
      commitment: 'singleGossip',
      preflightCommitment: 'singleGossip',
    },
  );

  const accountInfo = await connection.getAccountInfo(walletPubkey);
  if (accountInfo === null) {
    throw 'Error: cannot find the wallet account';
  }
  const info = walletAccountDataLayout.decode(Buffer.from(accountInfo.data));

  owners = owners.filter(
    owner =>
      info.owners
        .slice(0, info.n_owners)
        .findIndex(
          (realOwner: any) =>
            new PublicKey(realOwner.pubkey).toBase58() ===
            owner.publicKey.toBase58(),
        ) !== -1,
  );

  await storeCurrentData();
}

/**
 * Say hello with contract wallet
 */
export async function sayHelloWithContractWallet(): Promise<void> {
  const signers = owners.length ? [owners[0], owners[2]] : [];

  console.log(
    `Saying hello to ${wallet2Pubkey.toBase58()} with contract wallet`,
  );
  const internalTransaction = Wallet.createHelloTransaction(
    programId,
    wallet2Pubkey,
    [
      {
        publicKey: programAddress,
        secretKey: programAddress.toBuffer(), // fake, not used
      },
      ...signers,
    ],
  );
  const transaction = await Wallet.createInvokeTransaction(
    programId,
    walletPubkey,
    payerAccount.publicKey,
    internalTransaction,
    signers,
  );

  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(transaction),
    [payerAccount, ...signers],
    {
      commitment: 'singleGossip',
      preflightCommitment: 'singleGossip',
    },
  );
}

/**
 * Report the number of times the greeted account has been said hello to
 */
export async function reportWallet(walletPubkey: PublicKey): Promise<void> {
  const accountInfo = await connection.getAccountInfo(walletPubkey);
  if (accountInfo === null) {
    throw 'Error: cannot find the wallet account';
  }
  const info = walletAccountDataLayout.decode(Buffer.from(accountInfo.data));

  console.log(
    `number of ${walletPubkey.toBase58()} owners: `,
    info.owners.length,
  );

  for (let i = 0; i < info.owners.length; i++) {
    if (info.owners[i].weight == 0) {
      break;
    }
    console.log(
      `key #${i}: {\n`,
      `pubkey: ${new PublicKey(info.owners[i].pubkey).toBase58()}\n`,
      `weight: ${String(info.owners[i].weight)}\n}`,
    );
  }
}

export async function reportWallets(): Promise<void> {
  await reportWallet(walletPubkey);
  await reportWallet(wallet2Pubkey);
}
