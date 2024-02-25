use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use miden_client::{
    client::{rpc::TonicRpcClient, Client},
    store::{accounts::AuthInfo, data_store::SqliteDataStore},
};
use miden_lib::{accounts::faucets::create_basic_fungible_faucet, AuthScheme};
use miden_objects::{
    accounts::{Account, AccountData},
    assets::TokenSymbol,
    crypto::dsa::rpo_falcon512::KeyPair,
    utils::serde::Deserializable,
    Felt,
};

/// Creates a Miden fungible faucet from arguments
pub fn create_fungible_faucet(
    token_symbol: &str,
    decimals: &u8,
    max_supply: &u64,
    client: &mut Client<TonicRpcClient, SqliteDataStore>,
) -> Result<Account, io::Error> {
    let token_symbol = TokenSymbol::new(token_symbol).expect("Failed to parse token_symbol.");

    // Instantiate init_seed
    let init_seed: [u8; 32] = [0; 32];

    // Instantiate keypair and authscheme
    let auth_seed: [u8; 40] = [0; 40];
    let keypair = KeyPair::from_seed(&auth_seed).expect("Failed to generate keypair.");
    let auth_scheme = AuthScheme::RpoFalcon512 {
        pub_key: keypair.public_key(),
    };

    let (account, account_seed) = create_basic_fungible_faucet(
        init_seed,
        token_symbol,
        *decimals,
        Felt::try_from(*max_supply).expect("Max_supply is outside of the possible range."),
        auth_scheme,
    )
    .expect("Failed to generate faucet account.");

    client
        .insert_account(&account, account_seed, &AuthInfo::RpoFalcon512(keypair))
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to insert account into client.",
            )
        })?;

    Ok(account)
}

/// Imports a Miden fungible faucet from a file
pub fn import_fungible_faucet(
    faucet_path: &PathBuf,
    client: &mut Client<TonicRpcClient, SqliteDataStore>,
) -> Result<Account, io::Error> {
    let path = Path::new(faucet_path);
    let mut file = File::open(path).expect("Failed to open file.");

    let mut contents = Vec::new();
    let _ = file.read_to_end(&mut contents);

    let account_data =
        AccountData::read_from_bytes(&contents).expect("Failed to deserialize faucet from file.");

    client.import_account(account_data.clone()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Failed to import account into client.",
        )
    })?;

    Ok(account_data.account)
}
