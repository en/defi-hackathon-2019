use clap::load_yaml;
use codec::{Decode, Encode};
use dot_runtime::{self, template::RawEvent as DotEvent};
use executor::native_executor_instance;
use futures::stream::Stream;
use futures::Future;
use keyring::AccountKeyring;
use multidao_runtime::multidao::Identifier;
use node_primitives::{Hash, Index};
use primitives::{
    hexdisplay::HexDisplay,
    sr25519,
    storage::{well_known_keys, StorageKey},
    Pair,
};
use substrate_subxt::{
    balances::Balances,
    dot::{Dot, DotXt},
    ibc::{Ibc, IbcXt},
    system::System,
    Client, ClientBuilder,
};
use tokio::runtime::TaskExecutor;
use url::Url;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml)
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    execute(matches)
}

fn print_usage(matches: &clap::ArgMatches) {
    println!("{}", matches.usage());
}

fn update_client(executor: TaskExecutor, addr: Url, header: Vec<u8>) {
    let signer = AccountKeyring::Charlie.pair();
    let packet = ClientBuilder::<MultiDAORuntime>::new()
        .set_url(addr.clone())
        .build()
        .and_then(|client| client.xt(signer, None))
        .and_then(|xt| xt.ibc(|calls| calls.update_client(1, header)).submit())
        .map(|_| ())
        .map_err(|e| println!("{:?}", e));

    executor.spawn(packet);
}

fn execute(matches: clap::ArgMatches) {
    let password = matches.value_of("password");
    match matches.subcommand() {
        ("set-heads", Some(matches)) => {
            let suri = matches
                .value_of("suri")
                .expect("secret URI parameter is required; thus it can't be None; qed");
            let _pair = sr25519::Pair::from_string(suri, password).expect("Invalid phrase");

            let index = matches
                .value_of("nonce")
                .expect("nonce is required; thus it can't be None; qed");
            let _index = str::parse::<Index>(index)
                .expect("Invalid 'nonce' parameter; expecting an integer.");

            let genesis_hash = matches
                .value_of("genesis")
                .expect("genesis is required; thus it can't be None; qed");
            let genesis_hash: Hash = hex::decode(genesis_hash)
                .ok()
                .and_then(|x| Decode::decode(&mut &x[..]).ok())
                .expect("Invalid genesis hash");

            println!(
                "Using a genesis hash of {}",
                HexDisplay::from(&genesis_hash.as_ref())
            );
        }
        ("run", Some(matches)) => {
            let addr1 = matches
                .value_of("addr1")
                .expect("The address of chain dot is required; thus it can't be None; qed");
            let addr1 = Url::parse(&format!("ws://{}", addr1)).expect("Is valid url; qed");
            let addr2 = matches
                .value_of("addr2")
                .expect("The address of chain B is required; thus it can't be None; qed");
            let addr2 = Url::parse(&format!("ws://{}", addr2)).expect("Is valid url; qed");

            let mut rt = tokio::runtime::Runtime::new().unwrap();
            let executor = rt.executor();
            let executor1 = executor.clone();
            let addr2_1 = addr2.clone();

            let client_future = ClientBuilder::<DotRuntime>::new().set_url(addr1).build();
            let client = rt.block_on(client_future).unwrap();

            let stream = rt.block_on(client.subscribe_finalized_blocks()).unwrap();
            let blocks = stream.for_each(move |block_header| {
                let header_number = block_header.number;
                let state_root = block_header.state_root;
                let block_hash = block_header.hash();
                println!("header_number: {:?}", header_number);
                println!("state_root: {:?}", state_root);
                println!("block_hash: {:?}", block_hash);
                update_client(executor1.clone(), addr2_1.clone(), block_header.encode());
                Ok(())
            });
            executor.spawn(blocks.map_err(|_| ()));

            type EventRecords =
                Vec<srml_system::EventRecord<dot_runtime::Event, <DotRuntime as System>::Hash>>;

            let stream = rt.block_on(client.subscribe_events()).unwrap();
            let block_events =
                stream
                    .for_each(move |change_set| {
                        change_set
                            .changes
                            .iter()
                            .filter_map(|(_key, data)| {
                                data.as_ref().map(|data| Decode::decode(&mut &data.0[..]))
                            })
                            .filter_map(|result: Result<EventRecords, codec::Error>| result.ok())
                            .for_each(|events| {
                                events.into_iter().for_each(|event| match event.event {
                                    dot_runtime::Event::template(
                                        DotEvent::DotTransferred(who, amount, proof_height),
                                    ) => {
                                        let block_hash = change_set.block.clone();
                                        println!(
                                            "block_hash: {:?}, who: {:?}, amount: {:?}, proof_height: {:?}",
                                            block_hash, who, amount, proof_height);

                                        let addr2_2 = addr2.clone();
                                        let executor2 = executor.clone();
                                        let read_proof = client
                                            .read_proof(
                                                Some(block_hash.clone()),
                                                StorageKey(well_known_keys::CODE.to_vec()),
                                            )
                                            .and_then(move |proof| {
                                                let signer = AccountKeyring::Bob.pair();
                                                let packet = ClientBuilder::<MultiDAORuntime>::new()
                                                    .set_url(addr2_2.clone())
                                                    .build()
                                                    .and_then(|client| client.xt(signer, None))
                                                    .and_then(move |xt| {
                                                        xt.ibc(|calls| {
                                                            calls.recv_packet(
                                                                amount.encode(),
                                                                proof, proof_height
                                                            )
                                                        })
                                                        .submit()
                                                    })
                                                    .map(|_| ())
                                                    .map_err(|e| println!("{:?}", e));
                                                executor2.spawn(packet);
                                                Ok(())
                                            });
                                        executor.spawn(
                                            read_proof.map(|_| ()).map_err(|e| println!("{:?}", e)),
                                        );
                                    }
                                    _ => {}
                                });
                            });
                        Ok(())
                    })
                    .map_err(|e| println!("{:?}", e));
            rt.spawn(block_events);
            rt.shutdown_on_idle().wait().unwrap();
        }
        ("interchain-message", Some(matches)) => {
            let para_id = matches
                .value_of("para-id")
                .expect("para-id is required; thus it can't be None; qed");
            let para_id = str::parse::<u32>(para_id)
                .expect("Invalid 'para-id' parameter; expecting an integer.");

            let message = matches
                .value_of("message")
                .expect("message is required; thus it can't be None; qed");
            let message: Vec<u8> = hex::decode(message).expect("Invalid message");

            let (mut rt, client) = setup();

            let signer = AccountKeyring::Bob.pair();
            let xt = rt.block_on(client.xt(signer, None)).unwrap();

            let transfer_dot = xt.ibc(|calls| calls.transfer_dot(0)).submit();
            rt.block_on(transfer_dot).unwrap();
        }
        _ => print_usage(&matches),
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct DotRuntime;

impl System for DotRuntime {
    type Index = <dot_runtime::Runtime as srml_system::Trait>::Index;
    type BlockNumber = <dot_runtime::Runtime as srml_system::Trait>::BlockNumber;
    type Hash = <dot_runtime::Runtime as srml_system::Trait>::Hash;
    type Hashing = <dot_runtime::Runtime as srml_system::Trait>::Hashing;
    type AccountId = <dot_runtime::Runtime as srml_system::Trait>::AccountId;
    type Address = srml_indices::address::Address<Self::AccountId, u32>;
    type Header = <dot_runtime::Runtime as srml_system::Trait>::Header;
}

impl Balances for DotRuntime {
    type Balance = u64;
}

impl Dot for DotRuntime {}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MultiDAORuntime;

impl System for MultiDAORuntime {
    type Index = <multidao_runtime::Runtime as srml_system::Trait>::Index;
    type BlockNumber = <multidao_runtime::Runtime as srml_system::Trait>::BlockNumber;
    type Hash = <multidao_runtime::Runtime as srml_system::Trait>::Hash;
    type Hashing = <multidao_runtime::Runtime as srml_system::Trait>::Hashing;
    type AccountId = <multidao_runtime::Runtime as srml_system::Trait>::AccountId;
    type Address = srml_indices::address::Address<Self::AccountId, u32>;
    type Header = <multidao_runtime::Runtime as srml_system::Trait>::Header;
}

impl Balances for MultiDAORuntime {
    type Balance = u64;
}

impl Ibc for MultiDAORuntime {}

fn setup() -> (tokio::runtime::Runtime, Client<DotRuntime>) {
    env_logger::try_init().ok();
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let client_future = ClientBuilder::<DotRuntime>::new().build();
    let client = rt.block_on(client_future).unwrap();
    (rt, client)
}
