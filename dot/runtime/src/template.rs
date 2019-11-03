/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs
use support::{decl_event, decl_module, decl_storage, dispatch::Result};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait {
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        // 比如说要我生成一个proof，是proof用户Foo转移了100DOT,
        // 在我的这个简化的例子里就是生成一个指定高度(proof_height)的
        // EscrowAccount里有(Foo, 100)这个键值对
        EscrowAccount: map T::AccountId => Option<u128>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        // 用户调用这个extrinsic发起一笔跨链转账，目的是将数量为amount的DOT转移到cdai链上
        // 我这个地方忽略了目标链和目标链上的账户
        pub fn transfer_dot(origin, amount: u128) -> Result {
            let who = ensure_signed(origin)?;

            // 实际这个链在启动的时候应该创建一个链才有权限取钱的EscrowAccount，类型为T::AccountId
            // 用户转账就是将资产transfer到这个EscrowAccount里锁定起来
            // 我这个地方也简化成了一个map表示，key为谁，value为锁定了多少钱
            <EscrowAccount<T>>::insert(&who, amount);

            // 这个地方我获取了区块高度，并包含在了event里，relayer需要这个高度去获取proof
            let proof_height = <system::Module<T>>::block_number();

            Self::deposit_event(RawEvent::DotTransferred(who, amount, proof_height));
            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        BlockNumber = <T as system::Trait>::BlockNumber,
    {
        DotTransferred(AccountId, u128, BlockNumber),
    }
);

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::H256;
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        weights::Weight,
        Perbill,
    };
    use support::{assert_ok, impl_outer_origin, parameter_types};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type WeightMultiplierUpdate = ();
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }
    impl Trait for Test {
        type Event = ();
    }
    type TemplateModule = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities {
        system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap()
            .into()
    }

    #[test]
    fn it_works_for_default_value() {
        new_test_ext().execute_with(|| {
            // Just a dummy test for the dummy funtion `do_something`
            // calling the `do_something` function with a value 42
            assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
            // asserting that the stored value is equal to what we stored
            assert_eq!(TemplateModule::something(), Some(42));
        });
    }
}
