/**
 * Hello world
 */

import {
  establishConnection,
  establishPayer,
  loadProgram,
  sayHello,
  addOwner,
  removeOwner,
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
  await addOwner(100);

  // Say hello to an account
  await sayHello();

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
