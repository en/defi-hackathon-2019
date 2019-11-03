// Copyright 2017-2019 @polkadot/app-123code authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import BN from 'bn.js';
import React, { useState } from 'react';
import { Button, InputAddress, InputAddressSimple, InputNumber, TxButton } from '@polkadot/react-components';
import './style.css';

import Summary from './Summary';

interface Props {
  accountId?: string | null;
}

export default function Transfer (): React.ReactElement<Props> {
  const [amount, setAmount] = useState(undefined);
  const [recipientId, setRecipientId] = useState<string | null>(null);

  return (
    <section>
      <h1>transfer</h1>
      {/* <div className='ui--row hidden' >
        <AccountSelector onChange={setAccountId} />
      </div> */}
      <div className='ui--row'>
        <div >
          <h2 className="title">Crosschain Transfer Dot</h2>
          {/* <InputAddress
            label='recipient address for this transfer'
            onChange={setRecipientId}
            type='all'
          /> */}
          {/* <InputNumber label='amount to transfer' onChange={setAmount}/> */}

          <div style={{display:'flex', justifyContent:'space-between'}}>

          <div style={{
              height: '55px',
              marginTop: '1.25rem',
              background: 'white',
              borderRadius: '0.2rem',
              verticalAlign: 'middle',
              width: '49%'
            }}>
              <div className='input-row' style={{
                paddingLeft: '15px',
                height: '55px',
                display: 'inline-block',
                verticalAlign: 'middle',
                width: '60%'
              }}>
                {/* <label>
                  Transfer amount
                </label> */}
                <input className="number-input" type="number" onChange={(e)=>setAmount(e.target.value)}/>
              </div>

            </div>
            <div style={{width:'50%', marginTop:'1.25rem', background:'white'} }>
            <InputAddress
            defaultValue={'5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'}
            isDisabled={true}
            label={'Send to MultiDao account'}
            onChange={setRecipientId}
            type='account'
          />
            </div>
          
          </div>
            {/* <Summary className="sum">
              <p>
              To Chain : MultiDao
              </p>
              <p>
              To Account : Alice
              </p>
            </Summary> */}
          <Button.Group>
            <TxButton
              className='btn'
              accountId={'5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'}
              icon='send'
              label='MAKE TRANSFER'
              params={[amount]}
              tx='templateModule.transferDot'
            />
          </Button.Group>
        </div>
      </div>
    </section>
  );
}
