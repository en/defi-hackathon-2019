// Copyright 2017-2019 @polkadot/react-signer authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import { I18nProps } from '@polkadot/react-components/types';
import { QueueTx } from '@polkadot/react-components/Status/types';

import BN from 'bn.js';
import React from 'react';
import { GenericCall } from '@polkadot/types';
import { Call, InputAddress, Modal } from '@polkadot/react-components';

import Checks from './Checks';
import translate from './translate';

interface Props extends I18nProps {
  children?: React.ReactNode;
  hideDetails?: boolean;
  isSendable: boolean;
  tip?: BN;
  value: QueueTx;
}

function Transaction ({ children, hideDetails, isSendable, value: { accountId, extrinsic, isUnsigned }, t, tip }: Props): React.ReactElement<Props> | null {
  if (!extrinsic) {
    return null;
  }

  const { meta, method, section } = GenericCall.findFunction(extrinsic.callIndex);

  return (
    <div >
      <Modal.Header style={{height:'60px', padding:'15px', fontWeight:'400'}}>
        {section}.{method}
        {/* <label><details><summary>{
          meta && meta.documentation
            ? meta.documentation.join(' ')
            : ''
        }</summary></details></label> */}
      </Modal.Header>
      <Modal.Content className='ui--signer-Signer-Content' style={{paddingRight:'15px'}}>
        {!hideDetails && (
          <>
            {/* {!isUnsigned && accountId && (
              <InputAddress
                className='full'
                defaultValue={accountId}
                isDisabled
                isInput
                label={t('sending from my account')}
                withLabel
              />
            )} */}
            <Call value={extrinsic} />
            {!isUnsigned && (
              <Checks
                accountId={accountId}
                extrinsic={extrinsic}
                isSendable={isSendable}
                tip={tip}
              />
            )}
          </>
        )}
        {children}
      </Modal.Content>
    </div>
  );
}

export default translate(Transaction);
