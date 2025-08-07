const { ApiPromise, WsProvider } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');
const { ContractPromise } = require('@polkadot/api-contract');
const fs = require('fs');

async function main() {
  // --- Configuration ---
  const wsProvider = new WsProvider('wss://rpc1.paseo.popnetwork.xyz/');
  const api = await ApiPromise.create({ provider: wsProvider });

  const CONTRACT_ADDRESS = '5F2edUrKTZ67sWAB2GEUdvM1oqyH5Vj6W8wK5GDWsLPTR6sA';
  const SEED_PHRASE = 'prevent desert panic space kangaroo state vocal sauce there slice rural tuition';
  
  // Load contract metadata (ABI)
  const metadata = JSON.parse(fs.readFileSync('./target/ink/resource_registry/resource_registry.json', 'utf8'));
  
  // Create the contract instance
  const contract = new ContractPromise(api, metadata, CONTRACT_ADDRESS);

  // --- Prepare the arguments ---
  const keyring = new Keyring({ type: 'sr25519' });
  const user = keyring.addFromUri(SEED_PHRASE);

  const deviceMetadata = {
    device_type: { SmartPlug: null },
    capacity_watts: 2000,
    location: 'Living Room',
    manufacturer: 'PowerGrid Inc',
    model: 'SmartNode-1',
    firmware_version: '1.0',
    installation_date: 1754580839000,
  };
  
  const stakeAmount = '100000000000000000000'; // 100 PGT

  console.log('Submitting the transaction to register the device...');
  
  // --- Estimate gas and send the transaction ---
  const { gasRequired } = await contract.query.registerDevice(user.address, {}, deviceMetadata);
  
  const tx = contract.tx.registerDevice({ value: stakeAmount, gasLimit: gasRequired }, deviceMetadata);

  await tx.signAndSend(user, ({ status, events = [] }) => {
      if (status.isInBlock) {
        console.log(`Transaction included in block hash ${status.asInBlock}`);
        events.forEach(({ event: { data, method, section } }) => {
          console.log(`\t'${section}.${method}':: ${data.toString()}`);
        });
      } else if (status.isFinalized) {
        console.log('Transaction finalized successfully.');
        api.disconnect();
      }
    });
}

main().catch((error) => {
  console.error("An error occurred:", error);
  process.exit(-1);
});