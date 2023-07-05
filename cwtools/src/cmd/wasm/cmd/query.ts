// @ts-nocheck

import * as cosmwasm from '@cosmjs/cosmwasm-stargate';
import { Argv } from 'yargs';

export default async (yargs: Argv) => {
  const { argv } = yargs
    // positionals are arguments whose tag (--help or -h) can be omitted
    .positional('address', {
      describe: 'the smart contract address',
      type: 'string'
    })
    // options are arguments whose tag cannot be omitted
    .option('amount', {
      type: 'string'
    });
  
  // argv underscore is a special field that only parses the positionals
  const [address] = argv._.slice(-1);
  // then the client awaits to connect to the RPC URL
  const client = await cosmwasm.CosmWasmClient.connect(process.env.RPC_URL);
  // after which, it will look at the input (it is the --input option)
  const input = argv.input.startsWith('{') ? JSON.parse(argv.input) : cosmwasm.fromBinary(argv.input);
  // and then it will wait to receive a response
  const queryResult = await client.queryContractSmart(address, input);
  console.log('query result: ');
  console.dir(queryResult, { depth: null });
};
