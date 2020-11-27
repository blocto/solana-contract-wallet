/**
 * Hello world
 */

import {
  establishConnection,
  establishPayer,
  loadProgram,
  addOwner,
  removeOwner,
  sayHello,
  sayHelloWithContractWallet,
  reportWallet,
} from './wallet_test';

async function main() {
  console.log("Let's say hello to a Solana account...");

  // Establish connection to the cluster
  await establishConnection();

  // Determine who pays for the fees
  await establishPayer();

  // Load the program if not already loaded
  await loadProgram();

  // Add a new owner to the wallet account
  await addOwner(1000);
  // await addOwner(100);

  // Say hello
  await sayHello();

  // Say hello from a contract wallet
  await sayHelloWithContractWallet();

  // Show wallet account status
  await reportWallet();

  // Remove an owner from the wallet account
  await removeOwner(1);

  // Show wallet account status
  await reportWallet();

  console.log('Success');
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  },
);
