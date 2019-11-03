// Copyright 2017-2019 @polkadot/apps-routing authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import { Routes } from './types';

import Multidao from '@polkadot/app-multidao';

export default ([
  {
    Component: Multidao,
    display: {
      isHidden: true,
      needsAccounts: true,
      needsApi: [
        'tx.balances.transfer'
      ]
    },
    i18n: {
      defaultValue: 'Multidao'
    },
    icon: 'th',
    name: 'multidao'
  }
] as Routes);
