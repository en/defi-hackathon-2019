// Copyright 2017-2019 @polkadot/app-storage authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import {ApiProps} from '@polkadot/react-api/types';
import {I18nProps} from '@polkadot/react-components/types';
import {TabItem} from '@polkadot/react-components/Tabs';

import React from 'react';
import {Route, Switch} from 'react-router';
import {Tabs} from '@polkadot/react-components';
import {withApi} from '@polkadot/react-api';

import Dot from './Dot';
import Atom from './Atom';
import './style.css'

interface Props extends ApiProps, I18nProps {
  basePath: string;
  accountId?: string;
}

interface State {
  items: TabItem[];
}

class Selection extends React.PureComponent<Props, State> {
  public constructor(props: Props) {
    super(props);


    this.state = {
      items: [
        {
          // isRoot: true,
          name: 'Atom',
          text:'Atom'
        },
        {
          // isRoot: true,
          name: 'Dot',
          text: 'Dot'
        },
        
      ]
    };
  }


  async componentDidMount() {
    const {api} = this.props
    const assetNum = await api.query.multiDao.nextAssetId()
  }

  public render(): React.ReactNode {
    const {basePath, accountId} = this.props;
    const {items} = this.state;
    console.log(basePath, 'uniswap  basePath')

    return (
      <>
        <div className="banner" style={{}}> {'MultiDAO'}</div>
        <header>
          <Tabs
            basePath={basePath}
            items={items}
          />
        </header>
        <Switch>
          {/* <Route path={`${basePath}`} render={() => (<Atom accountId={accountId}/>)}/>
          <Route render={() => (<Dot accountId={accountId}/>)}/> */}
          <Route path={`${basePath}/Atom`} component={Atom} />
          <Route component={Dot} />
        </Switch>
      </>
    );
  }
}

export default withApi(Selection);
