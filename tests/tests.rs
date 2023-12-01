use concordium_smart_contract_testing::*;
use ccdpiggybank::*;


// A test account
 const ALICE: AccountAddress = AccountAddress([0u8; 32]);
 const JAMES: AccountAddress = AccountAddress([1u8; 32]);

// /// The initial balance of the ALICE test account.
 const ACC_INITIAL_BALANCE: Amount = Amount::from_ccd(10000);

// /// A [`Signer`] with one set of keys, used for signing transactions.
 const SIGNER: Signer = Signer::with_one_key();


 fn setup_chain_and_contract() -> (Chain, ContractInitSuccess) {
    let mut chain = Chain::new();

    chain.create_account(Account::new(ALICE, ACC_INITIAL_BALANCE));
    chain.create_account(Account::new(JAMES, ACC_INITIAL_BALANCE));

    let module = module_load_v1("ccdpiggybank.wasm.v1").expect("Module is valid and exists");
    let deployment = chain.module_deploy_v1(SIGNER,ALICE , module).expect("Deploying valid module should succeed");

    let initialization = chain
    .contract_init(
        SIGNER,
        ALICE,
        Energy::from(10000),
        InitContractPayload {
            mod_ref: deployment.module_reference,
            init_name: OwnedContractName::new_unchecked("init_ccdpiggybank".to_string()),
            param: OwnedParameter::empty(),
            amount: Amount::zero(),
        }
    )
    .expect("Initialization should always succeed"); 

    (chain, initialization)
}



#[test]
fn test_init(){
    let (chain, initialization) = setup_chain_and_contract();

    assert_eq!(
        chain.contract_balance(initialization.contract_address),
        Some(Amount::zero()),
        "Piggy bank is not initialized with balance of zero"
    );
   
}

#[test]
fn test_insert_intact() {
    let (mut chain, initialization) = setup_chain_and_contract();
    let insert_amount = Amount::from_ccd(10);

    let update = chain
    .contract_update(
       SIGNER,
        ALICE,
        Address::Account(ALICE),
        Energy::from(10000),
        UpdateContractPayload {
            amount: insert_amount,
            address: initialization.contract_address,
            receive_name: OwnedReceiveName::new_unchecked("ccdpiggybank.insert".to_string()),
            message: OwnedParameter::empty(),
        },
    );

    assert!(update.is_ok(), "Inserting into intact piggy bank failed");
    assert_eq!(
        chain.contract_balance(initialization.contract_address),
        Some(insert_amount),
        "Piggy bank balance does not match amount inserted"
    );
}

#[test]
fn test_smash_intact_not_owner() {
   let (mut chain, initialization) = setup_chain_and_contract();

    let update_err = chain
        .contract_update(
            SIGNER,
            ALICE,
            Address::Account(ALICE),
            Energy::from(10000),
            UpdateContractPayload {
                amount: Amount::zero(),
                address: initialization.contract_address,
                receive_name: OwnedReceiveName::new_unchecked("ccdpiggybank.smash".to_string()),
                message: OwnedParameter::empty(),
            },
        )
        .expect_err("Smashing should only succeed for the owner");

    let return_value = update_err
        .return_value()
        .expect("Contract should reject and thus return bytes");
    let error: SmashError = from_bytes(&return_value)
        .expect("Contract should return a `SmashError` in serialized form");

    assert_eq!(
        error,
        SmashError::NotOwner,
        "Contract did not fail due to a NotOwner error"
    );
    assert_eq!(
        chain.account_balance_available(ALICE),
        Some(ACC_INITIAL_BALANCE - update_err.transaction_fee),
        "The invoker account was incorrectly charged"
    )
}