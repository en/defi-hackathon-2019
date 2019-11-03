// Copyright 2017-2019 @polkadot/app-123code authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import React, { useEffect, useState } from 'react';
import styled from 'styled-components';
import { InputAddress } from '@polkadot/react-components';
import { AccountIndex, Balance, Nonce } from '@polkadot/react-query';
import Bubble from './Bubble'

interface Props {
  className?: string;
  onChange: (accountId: string | null) => void;
}

function AccountSelector ({ className, onChange }: Props): React.ReactElement<Props> {
  const [accountId, setAccountId] = useState<string | null>(null);

  useEffect((): void => onChange(accountId), [accountId]);

  return (
    <div className='ui--row' style={{display:'flex', justifyContent:'center'}}>
      <InputAddress
        style={{ width:'80%'}}
        label='my default account'
        onChange={setAccountId}
        type='account'
      /> 
      <span style={{ width:'20%'}} >
        <Bubble style={{borderRatio:'10px',width:'100%', height:'58px'}} color='grey' label='balance'>
          <Balance params={accountId} />
        </Bubble>
      </span>
      
    </div>
    
  );
}

export default styled(AccountSelector)`
  align-items: flex-end;

  .summary {
    text-align: center;
  }
`;
