/**
 * Hello world
 */

import {
  establishConnection,
  establishPayer,
  loadProgram,
  addOwner,
  addContractWalletAsOwner,
  removeOwner,
  sayHello,
  sayHelloWithContractWallet,
  reportWallets,
} from './wallet_test';

async function main() {
  console.log("Let's say hello to a Solana account...");

  // Establish connection to the cluster
  await establishConnection();

  // Determine who pays for the fees
  await establishPayer();

  // Load the program if not already loaded
  await loadProgram();

  // Add a new owner to the normal wallet account
  // await addOwner(1000);
  // await addOwner(100);

  // Add a normal wallet as owner of the contract controlled wallet account
  // await addContractWalletAsOwner();
  
  // Show wallet account status
  await reportWallets();

  // Say hello
  await sayHello();

  // Say hello from a contract wallet
  await sayHelloWithContractWallet();

  // Show wallet account status
  await reportWallets();

  // Remove an owner from the wallet account
  await removeOwner(1);

  // Show wallet account status
  await reportWallets();

  console.log('Success');
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  },
);
