#![allow(unused_imports)]
#![allow(unused_variables)]

use codec::{Decode, Encode};
use rstd::{convert::TryInto, prelude::*, result};
use sr_primitives::traits::{
    Bounded, CheckedDiv, CheckedSub, Hash, Member, One, Printable, SaturatedConversion,
    SimpleArithmetic, StaticLookup, Zero,
};
use sr_primitives::{print, traits::Header};
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, traits::Currency, Parameter,
    StorageMap, StorageValue,
};

use rstd::convert::Into;
use system::{ensure_root, ensure_signed};

use app_crypto::RuntimeAppPublic;
use system::offchain::SubmitSignedTransaction;

pub const KEY_TYPE: app_crypto::KeyTypeId = app_crypto::KeyTypeId(*b"ofcb");

// 这个是另外一条链的标识id
pub type Identifier = u32;

// 这个结构保存还没有处理过的跨链消息和它的proof
#[derive(Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
struct UncheckedPacket {
    packet: Vec<u8>,
    proof: Vec<Vec<u8>>,
}

/// The type of requests we can send to the offchain worker
#[cfg_attr(feature = "std", derive(PartialEq, Eq, Debug))]
#[derive(Encode, Decode)]
pub enum OffchainRequest<T: system::Trait> {
    /// If an authorised offchain worker sees this ping, it shall respond with a `pong` call
    Ping(u64, <T as system::Trait>::AccountId),
}

pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// The units in which we record balances.
    type Balance: Member + Parameter + SimpleArithmetic + Default + Copy;
    /// The arithmetic type of asset identifier.
    type AssetId: Parameter + SimpleArithmetic + Default + Copy;

    /// A dispatchable call type. We need to define it for the offchain worker to
    /// reference the `pong` function it wants to call.
    type Call: From<Call<Self>>;

    /// Let's define the helper we use to create signed transactions with
    type SubmitTransaction: SubmitSignedTransaction<Self, <Self as Trait>::Call>;

    /// The local keytype
    type KeyType: RuntimeAppPublic + From<Self::AccountId> + Into<Self::AccountId> + Clone;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        <T as Trait>::Balance,
        <T as Trait>::AssetId {
        /// Some assets were issued.
        Issued(AssetId, AccountId, Balance),
        /// Some assets were transferred.
        Transferred(AssetId, AccountId, AccountId, Balance),
        /// Some assets were destroyed.
        Destroyed(AssetId, AccountId, Balance),
        /// Made CDP event
        MadeCDP(AccountId, AssetId, Balance, Balance),
        PacketReceived(Vec<u8>),

        /// When we received a Pong, we also Ack it.
        Ack(u64, AccountId),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Sfhkt
//    where
//        u64: core::convert::From<<T as Trait>::AssetId>,
//        u128: core::convert::From<<T as Trait>::Balance>,
//        <T as Trait>::Balance: core::convert::From<u128>
    {
        /// The next asset identifier up for grabs.
        NextAssetId get(next_asset_id): T::AssetId;
        /// The total unit supply of an asset.
        TotalSupply get(get_asset_total_supply): map T::AssetId => T::Balance;
        /// The number of units of assets held by any given account.
        Balances get(get_asset_balance): map (T::AssetId, T::AccountId) => T::Balance;

        /// The default inherent asset in this platform
        InherentAsset get(inherent_asset_id): T::AssetId;
        ///
        CollateralRate get(rate) config(): u64;
        Prices get(get_asset_price): map T::AssetId => u64;

        /// The main repository of this project
        /// return: AssetId balance, cdai balance
        CDP get(cdai_by_account_and_asset): map (T::AccountId, T::AssetId) => (T::Balance, T::Balance);
        /// for test only
        OwnerA get(owner_a) config(): T::AccountId;
        OwnerB get(owner_b) config(): T::AccountId;
        OwnerC get(owner_c) config(): T::AccountId;

        Plugin: map Identifier => Option<Vec<u8>>;
        Heads: map T::BlockNumber => Option<Vec<u8>>;
        PacketQueue: map T::BlockNumber => Option<Vec<UncheckedPacket>>;

        /// Requests made within this block execution
        OcRequests get(oc_requests): Vec<OffchainRequest<T>>;
        /// The current set of keys that may submit pongs
        Authorities get(authorities): Vec<T::AccountId>;
    }

    add_extra_genesis {
        config(assets): Vec<(T::AccountId, T::Balance)>;
        config(prices): Vec<u64>;

        build(|config: &GenesisConfig<T>| {
            for asset in config.assets.iter() {
                let (account, amount) = asset;
                <Module<T>>::_issue(account.clone(), amount.clone());
                let to_account = <OwnerB<T>>::get();
                let asset_id = <NextAssetId<T>>::get() - 1.into();
                <Module<T>>::transfer(account.clone(), asset_id, to_account, 50000.into());
                let to_account = <OwnerC<T>>::get();
                let asset_id = <NextAssetId<T>>::get() - 1.into();
                <Module<T>>::transfer(account.clone(), asset_id, to_account, 50000.into());
            }

            for (i, price) in config.prices.iter().enumerate() {
                let asset_id = (i as u128).saturated_into::<T::AssetId>();
                <Prices<T>>::insert(&asset_id, price);
            }
        })
    }

}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call
    where origin: T::Origin
    //          u64: core::convert::From<<T as Trait>::AssetId>,
    //          u128: core::convert::From<<T as Trait>::Balance>,
    //          <T as Trait>::Balance: core::convert::From<u128>
    {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        /// Clean the state on initialisation of a block
        fn on_initialize(_now: T::BlockNumber) {
            // At the beginning of each block execution, system triggers all
            // `on_initialize` functions, which allows us to set up some temporary state or - like
            // in this case - clean up other states
            <Self as Store>::OcRequests::kill();
        }

        /// Issue a new class of fungible assets. There are, and will only ever be, `total`
        /// such assets and they'll all belong to the `origin` initially. It will have an
        /// identifier `AssetId` instance: this will be specified in the `Issued` event.
        /// This will make a increased id asset.
        /// @origin
        /// @total    How much balance of new asset
        fn issue(origin, total: T::Balance) -> Result {
            let origin = ensure_signed(origin)?;

            Self::_issue(origin, total)
        }

        /// Destroy any assets of `id` owned by `origin`.
        /// @origin
        /// @id      Asset id to be destroyed
        fn destroy(origin, id: T::AssetId) -> Result {
            let origin = ensure_signed(origin)?;
            let balance = <Balances<T>>::take((id, origin.clone()));
            ensure!(!balance.is_zero(), "origin balance should be non-zero");

            <TotalSupply<T>>::mutate(id, |total_supply| *total_supply -= balance);
            Self::deposit_event(RawEvent::Destroyed(id, origin, balance));

            Ok(())
        }

        /// Transfer an asset to another account
        pub fn transfer_asset(origin,
                    id: T::AssetId,
                    to_account: T::AccountId,
                    amount: T::Balance
        ) -> Result {
            let from_account = ensure_signed(origin)?;
            Self::transfer(from_account, id, to_account, amount);

            Ok(())
        }


        /// Set global collateral rate
        /// @origin
        /// @rate    the global fee rate on each transaction, multi 100x
        pub fn set_collateral_rate(origin, rate: u64) -> Result {
            ensure_signed(origin)?;
            <CollateralRate>::mutate(|cr| *cr = rate);

            Ok(())
        }

        // price = a, a = dollar value * 10000
        pub fn set_price(origin,asset_id: T::AssetId, price: u64) -> Result {
            ensure_signed(origin)?;
            <Prices<T>>::insert(&asset_id, price);

            Ok(())
        }


        pub fn make_cdp(origin, asset: T::AssetId, amount: T::Balance) -> Result {
            let account = ensure_signed(origin)?;

            let rate: u64 = Self::rate();
            let amount_u128 = amount.saturated_into::<u128>();
            let price = Self::get_asset_price(asset.clone());
            print(price);
            let cdai_amount_u128: u128 = amount_u128 * (price as u128) / (rate as u128 * 100);
            let cdai_amount = cdai_amount_u128.saturated_into::<T::Balance>();
            let r = Self::cdai_by_account_and_asset((account.clone(), asset.clone()));
            let (old_asset_amount, old_cdai_amount) = r;

            let tp: u128 = old_asset_amount.saturated_into::<u128>();
            print(tp as u64);
            let tp: u128 = old_cdai_amount.saturated_into::<u128>();
            print(tp as u64);
            print(amount_u128 as u64);
            print(cdai_amount_u128 as u64);

            <CDP<T>>::insert((account.clone(), asset.clone()), (old_asset_amount + amount, old_cdai_amount + cdai_amount));

            // need reduce account balance
            let account_key = (asset.clone(), account.clone());
            let origin_balance = <Balances<T>>::get(&account_key);
            let tp: u128 = origin_balance.saturated_into::<u128>();
            print(tp as u64);
            <Balances<T>>::insert(account_key, origin_balance - amount);

            let inherent_asset_id = Self::inherent_asset_id();
            let account_key = (inherent_asset_id.clone(), account.clone());
            let origin_balance = <Balances<T>>::get(&account_key);
            <Balances<T>>::insert(account_key, origin_balance + cdai_amount);

            Self::deposit_event(RawEvent::MadeCDP(account, asset, amount, cdai_amount));

            //let asset_id = 1u128.saturated_into::<T::AssetId>();

            Ok(())
        }

        // 这个是处理跨链前的第一步，cdai需要拥有所有跨链原链的消息验证插件
        // 也就是sf/dot-plugin这个wasm
        fn initialize_plugin(origin, id: Identifier, plugin: Vec<u8>) -> Result {
             ensure_root(origin)?;
             <Plugin>::insert(id, plugin);
             Ok(())
         }

        // relayer不停的relay header过来
        fn update_client(origin, id: Identifier, header: Vec<u8>) -> Result {
            let who = ensure_signed(origin)?;
            // decode了header获取它的高度保存起来，这个地方实际需要区分哪个链的header
            // 暂时忽略了
            let h: <T as system::Trait>::Header = Decode::decode(&mut &header[..]).expect("todo: handle error");
            // 保存前应该验证header，这个地方暂时忽略，估计hackathon时要补上
            <Heads<T>>::insert(h.number(), header);
            // 这个地方是个hack，header和packet是异步发过来的，但是目前来看顺序都是收到异步消息然后收到它的header，于是从queue里取出packet做验证在正常情况下都是可行的
            if let Some(ups) = <PacketQueue<T>>::get(h.number()) {
                print("run_wasm");
                // TODO: 0 for dot chain, 1 for atom chain
                let plugin = <Plugin>::get(0).expect("plugin must be exist");
                for up in ups.iter() {
                    if wasm_proof::run_wasm(&plugin) {
                        print("mint token");
                        let amount: u128 = Decode::decode(&mut &up.packet[..]).expect("can not decode packet");
                        // TODO: 2 for DOT, 1 for ATOM
                        let asset_id = 2.saturated_into::<T::AssetId>();
                        let amount = amount.saturated_into::<T::Balance>();
                        Self::mint_asset(who.clone(), asset_id, amount);
                    }
                }
                <PacketQueue<T>>::remove(h.number());
                // 验完，执行结果，比如查到消息里转来了多少dot，然后mint出对应的dot'
                // 之后就用dot'换cdai了
            }
            Ok(())
        }

        // 这个地方是收到了跨链消息，因为现在我试验收到消息时,header还没过来，所以这个地方只塞到了queue里，没有验消息，实际上应该查下对应的头存不存在，如果存在就验证，并删除消息
        fn recv_packet(origin, packet: Vec<u8>, proof: Vec<Vec<u8>>, proof_height: T::BlockNumber) -> Result {
            ensure_signed(origin)?;
            // TODO
            let _ = <PacketQueue<T>>::append(proof_height, &[UncheckedPacket{packet: packet.clone(), proof: proof}]);
            Self::deposit_event(RawEvent::PacketReceived(packet));
            Ok(())
        }


        /// The entry point function: storing a `Ping` offchain request with the given `nonce`.
        pub fn ping(origin, nonce: u64) -> Result {
            // It first ensures the function was signed, then it store the `Ping` request
            // with our nonce and author. Finally it results with `Ok`.
            let who = ensure_signed(origin)?;

            <Self as Store>::OcRequests::mutate(|v| v.push(OffchainRequest::Ping(nonce, who)));
            Ok(())
        }

        /// Called from the offchain worker to respond to a ping
        pub fn pong(origin, nonce: u64) -> Result {
            // We don't allow anyone to `pong` but only those authorised in the `authorities`
            // set at this point. Therefore after ensuring this is signed, we check whether
            // that given author is allowed to `pong` is. If so, we emit the `Ack` event,
            // otherwise we've just consumed their fee.
            let author = ensure_signed(origin)?;

            if Self::is_authority(&author) {
                // here, to set asset price

                Self::deposit_event(RawEvent::Ack(nonce, author));
            }

            Ok(())
        }

        // Runs after every block within the context and current state of said block.
        fn offchain_worker(_now: T::BlockNumber) {
            if let Some(key) = Self::authority_id() {
                Self::offchain(&key);
            }
        }

        // Simple authority management: add a new authority to the set of keys that
        // are allowed to respond with `pong`.
        pub fn add_authority(origin, who: T::AccountId) -> Result {
            // In practice this should be a bit cleverer, but for this example it is enough
            // that this is protected by a root-call (e.g. through governance like `sudo`).
            let _me = ensure_root(origin)?;

            if !Self::is_authority(&who){
                <Authorities<T>>::mutate(|l| l.push(who));
            }

            Ok(())
        }

    }
}

impl<T: Trait> Module<T>
//where
//    u64: core::convert::From<<T as Trait>::AssetId>,
//    u128: core::convert::From<<T as Trait>::Balance>,
//    <T as Trait>::Balance: core::convert::From<u128>,
{
    /// Issue a new class of fungible assets. There are, and will only ever be, `total`
    /// such assets and they'll all belong to the `origin` initially. It will have an
    /// identifier `AssetId` instance: this will be specified in the `Issued` event.
    /// This will make a increased id asset.
    /// @origin
    /// @total    How much balance of new asset
    fn _issue(account: T::AccountId, total: T::Balance) -> rstd::result::Result<(), &'static str> {
        let id = Self::next_asset_id();
        <NextAssetId<T>>::mutate(|id| *id += One::one());

        <Balances<T>>::insert((id, account.clone()), total);
        <TotalSupply<T>>::insert(id, total);

        Self::deposit_event(RawEvent::Issued(id, account, total));

        Ok(())
    }

    /// Move some assets from one holder to another.
    /// @from_account    The account lost amount of a certain asset balance
    /// @id              The asset id to be transfered
    /// @to_account      The account receive the sent asset balance
    /// @amount          The amount value to be transfered
    fn transfer(
        from_account: T::AccountId,
        id: T::AssetId,
        to_account: T::AccountId,
        amount: T::Balance,
    ) -> rstd::result::Result<(), &'static str> {
        let origin_account = (id, from_account.clone());
        let origin_balance = <Balances<T>>::get(&origin_account);
        let target = to_account;
        ensure!(!amount.is_zero(), "transfer amount should be non-zero");
        ensure!(
            origin_balance >= amount,
            "origin account balance must be greater than or equal to the transfer amount"
        );

        Self::deposit_event(RawEvent::Transferred(
            id,
            from_account,
            target.clone(),
            amount,
        ));
        <Balances<T>>::insert(origin_account, origin_balance - amount);
        <Balances<T>>::mutate((id, target), |balance| *balance += amount);

        Ok(())
    }

    /// The main entry point, called with account we are supposed to sign with
    fn offchain(key: &T::AccountId) {
        for e in <Self as Store>::OcRequests::get() {
            match e {
                OffchainRequest::Ping(nonce, _who) => Self::respond(key, nonce), // there would be potential other calls
            }
        }
    }

    /// Responding to as the given account to a given nonce by calling `pong` as a
    /// newly signed and submitted trasnaction
    fn respond(key: &T::AccountId, nonce: u64) {
        runtime_io::print_utf8(b"Received ping, sending pong");
        let call = Call::pong(nonce);
        let _ = T::SubmitTransaction::sign_and_submit(call, key.clone().into());
    }

    /// Helper that confirms whether the given `AccountId` can sign `pong` transactions
    fn is_authority(who: &T::AccountId) -> bool {
        Self::authorities().into_iter().find(|i| i == who).is_some()
    }

    /// Find a local `AccountId` we can sign with, that is allowed to `pong`
    fn authority_id() -> Option<T::AccountId> {
        // Find all local keys accessible to this app through the localised KeyType.
        // Then go through all keys currently stored on chain and check them against
        // the list of local keys until a match is found, otherwise return `None`.
        let local_keys = T::KeyType::all()
            .iter()
            .map(|i| (*i).clone().into())
            .collect::<Vec<T::AccountId>>();

        Self::authorities().into_iter().find_map(|authority| {
            if local_keys.contains(&authority) {
                Some(authority)
            } else {
                None
            }
        })
    }

    fn mint_asset(who: T::AccountId, asset_id: T::AssetId, amount: T::Balance) -> Result {
        // for demo, always use Alice
        let who = <OwnerA<T>>::get();
        print("in mint asset");
        
        let account_key = (asset_id.clone(), who.clone());
        let origin_balance = <Balances<T>>::get(&account_key);
        //let tp: u128 = origin_balance.saturated_into::<u128>();
        //print(tp as u64);
        <Balances<T>>::insert(account_key, origin_balance + amount);

        let origin_supply = <TotalSupply<T>>::get(&asset_id);
        <TotalSupply<T>>::insert(asset_id, origin_supply + amount);

        //Self::deposit_event(RawEvent::Issued(id, who, total));

        Ok(())
    }
}
