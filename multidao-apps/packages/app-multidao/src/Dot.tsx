// Copyright 2017-2019 @polkadot/app-123code authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import BN from 'bn.js';
import React from 'react';
import {withApi} from '@polkadot/react-api';
import {Button, Dropdown, TxButton, TxComponent} from '@polkadot/react-components';
import {ApiProps} from '@polkadot/react-api/types';
import _ from 'lodash';
import Decimal from 'decimal.js';
import options from './config';
import './style.css';
import Summary from './Summary';
import { ApiConsumer } from '@polkadot/react-api/ApiContext';

const accountId = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'

interface Props extends ApiProps {
  accountId?: string;
}

interface State {
  assetAmount?: number;
  inherentAmount?: number;
  assetId?: number;
  inherentBalance?: string;
  assetBalance?: string;
  address?: string;
  inherentAssetId?: number;
  rate: number;
  price: number;
}

class Dot extends TxComponent<Props, State> {
  public state: State = {
    assetId: 2,
    assetAmount: 0,
    inherentAmount: 0,
    assetBalance:undefined,
    inherentBalance:undefined,
    inherentAssetId: undefined,
    rate: 0,
    price: 0
  };

  async componentDidMount() {
   
    this.initData()
    
  }

  async initData() {
    const {api} = this.props
    let res = await api.query.multiDao.inherentAsset()
    const inherentAssetId: number = Number(res.toString())

    res = await api.query.multiDao.collateralRate()
    const rate = Number(res.toString())
    

    res = await api.query.multiDao.prices(2)
    let price = Number(res.toString())
    if(price>0){
      price = new Decimal(price).div(10000).toNumber()
    }
    const assetBalance = await this.getBalance(2)
    const inherentBalance = await this.getBalance(0)
    const state = {inherentAssetId, rate, price, assetBalance, inherentBalance}
    console.log(state)
    this.setState(state)
  }

  async getBalance(assetId:any) {
    const {api} = this.props

    const res = await api.query.multiDao.balances([assetId, accountId])
    const balance = res.toString()
    return balance
  }

  public render(): React.ReactNode {
    // const {accountId} = this.props;
    const {
      assetAmount, inherentAmount, assetId, inherentAssetId,
      inherentBalance, rate, assetBalance, price
    } = this.state;

    return (
      <section>
        {/* <h2 className="title">
          Dot
        </h2> */}
        <div className='ui--row center'>
          {/* <div style={{padding:'10%'}}> */}
          <div>

            {/* <div style={{
              background: 'white',
              borderRadius: '2rem',
              height: '55px'
            }}>
              <Dropdown
                dropdownClassName='asset-dropdown'
                value={formType}
                onChange={this.selectType}
                options={typeOptions}
              />
            </div> */}

            <div className="inputContainer">
              
              <div style={{
              height: '55px',
              width: '49%',
              verticalAlign: 'middle',
              borderRadius:'0.1rem'
              }}>
                <div className="label">
                {/*{formType === 1 ? 'inherent asset amount' : 'min inherent asset amount'}*/}
                How much {this.getAssetName(assetId)} would you like to collateralize?
              </div>
              <div style={{display:'flex', justifyContent:'space-between'}}>

                <div className='input-row' style={{
                  paddingLeft: '15px',
                  height: '55px',
                  display: 'inline-block',
                  verticalAlign: 'middle',
                  width: '60%',
                  background: 'white',

                }}>
                  <input className="number-input" type="number" value={assetAmount}
                        onChange={this.onChangeAssetAmount}/>
                </div>

                <Dropdown
                  style={{
                    display: 'inline-block',
                    height: '55px',
                    verticalAlign: 'middle',
                    background: 'white',
                    width: '40%',

                  }}
                  dropdownClassName='asset-dropdown'
                  value={2}
                  isDisabled={true}
                  // onChange={this.selectAsset}
                  options={options}
                />
                </div>
              </div>
              <div style={{
               height: '55px',
               width: '49%',
               verticalAlign: 'middle',
               borderRadius:'0.1rem'
              }}>
                 <div className="label">
                {/*{formType === 1 ? 'inherent asset amount' : 'min inherent asset amount'}*/}
                How much {this.getAssetName(inherentAssetId)} would you like to generate?
              </div>
              <div style={{display:'flex', justifyContent:'space-between'}}>

                <div className='input-row' style={{
                  paddingLeft: '15px',
                  height: '55px',
                  display: 'inline-block',
                  verticalAlign: 'middle',
                  width: '60%',
                  background: 'white',
                }}>
                  {/* <label>
                    Output amount
                  </label> */}
                  <input className="number-input" type="number" value={inherentAmount}
                        onChange={this.onChangeInherentAmount}/>
                </div>

                <Dropdown
                  style={{
                    display: 'inline-block',
                    height: '55px',
                    verticalAlign: 'middle',
                    background: 'white',
                    width: '40%',
                  }}
                  dropdownClassName='asset-dropdown'
                  value={0}
                  isDisabled={true}
                  // onChange={this.selectAsset}
                  options={options}
                />
                </div>
              </div>
            </div>
            
            <Summary style={{
               display:'flex',
               justifyContent:'space-around',
              background: 'rgba(41,44,47,.2)',
              marginTop: '1.25rem',
              boxShadow: '0 0 5px #ccc',
              borderRadius: '0.1rem',
              opacity: '1',
              fontSize:'12px',
              color:'#9aa3ad'
            }}>
              <div>
                <p>
                  {this.getAssetName(inherentAssetId)} Balance : {inherentBalance}
                </p>
                <p>
                  {this.getAssetName(assetId)} Balance : {assetBalance}
                </p>
              </div>
             <div>
             {price>0 && <p>
                  1 {this.getAssetName(assetId)} ≈ {price} USD
              </p>}
              {rate>0 && <p>
                CDP Rate : {rate}%
              </p>}
             </div>
              
            </Summary>

            <Button.Group>
              <TxButton
                accountId={accountId}
                className={'pool-button'}
                label='COLLATERALIZE & GENERATE CDAI'
                params={[assetId, assetAmount]}
                tx='multiDao.makeCdp'
                ref={this.button}
                onSuccess={this.onSuccess}
              /> 
            </Button.Group>


          </div>
        </div>
      </section>
    );
  }

  private getAssetName = (id?: number) => {
    const asset = options.find(asset => {
      return asset.value == id
    })
    return asset && asset.text ? asset.text : ''
  }

 

  private onSuccess = async (result: any) => {
    console.log(result, '===============1===============')
    this.initData()
  }


  private onChangeInherentAmount = (e: any): void => {
    let inherentAmount = e.target.value
    this.setState({inherentAmount});
    // const inherentBalance = this.state.inherentBalance;
    // const assetBalance = this.state.assetBalance;
    if (Number(inherentAmount) != 0 && Number(inherentAmount) != 0) {
      _.delay(() => this.calcRate('output'), 100)
    }
  }

  private onChangeAssetAmount = (e: any): void => {
    let assetAmount = e.target.value
    this.setState({assetAmount});
    // const inherentBalance = this.state.inherentBalance;
    // const assetBalance = this.state.assetBalance;
    if (Number(assetAmount) != 0 && Number(assetAmount) != 0) {
      _.delay(() => this.calcRate('input'), 100)
    }
  }

  // 获得费率
  private calcRate = async (type?: string) => {
    let {
      inherentAmount = -1,
      assetAmount = -1,
      assetId,
      price,
      rate,
    } = this.state;
    if (!assetId) {
      console.log('no asset selected');
      return
    }
    const {api} = this.props
    // input inherent asset
    if (_.eq(inherentAmount, '')) {
      inherentAmount = 0
    } else {
      inherentAmount = Number(inherentAmount)
    }
    if (_.eq(assetAmount, '')) {
      assetAmount = 0
    } else {
      assetAmount = Number(assetAmount)
    }
   
    const CDPRate = new Decimal(rate).div(100)
    if (type === 'input' && assetAmount > 0 && _.isNumber(assetAmount)) {
      const outputNum = new Decimal(assetAmount || 0).times(price).div(CDPRate).toFixed(8)
      console.log(outputNum, '===============')
      this.setState({
        inherentAmount: Number(outputNum)
      })
    } else if (type === 'output' && inherentAmount > 0 && _.isNumber(inherentAmount)) {
      const inputNum = new Decimal(inherentAmount || 0).div(price).times(CDPRate).toFixed(8)
      this.setState({
        assetAmount: Number(inputNum)
      })
    }

  }


}

export default withApi(Dot);
