/**
 * Execute wasm command.
 */

// @ts-nocheck

import * as cosmwasm from '@cosmjs/cosmwasm-stargate';
import { stringToPath } from '@cosmjs/crypto';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { decryptMnemonic } from '../../../common';
import { Argv } from 'yargs';

export default async (yargs: Argv) => {
  // firstly, we will parse this argument from the cmd using yargs import
  const { argv } = yargs
    // the positional is the address of the smart contract (as discussed)
    .positional('address', {
      describe: 'the smart contract address',
      type: 'string'
    })
    // the options include amount (i.e. for atomic-swap, we use this amount
    // to determine the sender's amount to offer swapping)
    .option('amount', {
      type: 'string'
    })
    // and a memo
    .option('memo', {
      type: 'string'
    });
  
  // the address (pattern matched)
  const [address] = argv._.slice(-1);
  const prefix = process.env.PREFIX || 'orai';
  const denom = process.env.DENOM || 'orai';
  // decrypt the mnemonic to ensure that the password is correct
  const mnemonic = argv.ENCRYPTED_MNEMONIC ? decryptMnemonic(argv.ENCRYPTED_MNEMONIC) : argv.MNEMONIC;
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    hdPaths: [stringToPath(process.env.HD_PATH)],
    prefix
  });
  const [firstAccount] = await wallet.getAccounts();

  const client = await cosmwasm.SigningCosmWasmClient.connectWithSigner(process.env.RPC_URL, wallet, {
    gasPrice: GasPrice.fromString(`${process.env.GAS_PRICES}${prefix}`),
    prefix
  });

  // then we parse the json input, which would be --input (argv.input equivalent)
  const input = JSON.parse(argv.input);
  // we parse the amount
  const amount = argv.amount ? [{ amount: argv.amount, denom }] : undefined;

  // and finally, the response is calling execute.ts script
  // specifically, it requires initiator's address, the contract's address, the parsed input, etc.
  // NOTE: these are the information stored in MessageInfo argument passed to the execute function
  const result = await client.execute(firstAccount.address, address, input, 'auto', argv.memo, amount);
  console.log('result: ', result);
};
