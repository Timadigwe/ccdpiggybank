use concordium_smart_contract_testing::*;
use ccdpiggybank::*;


// A test account
 const ALICE: AccountAddress = AccountAddress([0u8; 32]);
 const JAMES: AccountAddress = AccountAddress([1u8; 32]);

// /// The initial balance of the ALICE test account.
 const ACC_INITIAL_BALANCE: Amount = Amount::from_ccd(10_000);

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
fn test_smash_intact() {
    let (mut chain, initialization) = setup_chain_and_contract();

    let update = chain
        .contract_update(
            Signer::with_one_key(),
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
        .expect("Owner is allowed to smash intact piggy bank");

    let invoke = chain
        .contract_invoke(
            JAMES,
            Address::Account(JAMES),
            Energy::from(10000),
            UpdateContractPayload {
                amount: Amount::zero(),
                address: initialization.contract_address,
                receive_name: OwnedReceiveName::new_unchecked("ccdpiggybank.view".to_string()),
                message: OwnedParameter::empty(),
            },
        )
        .expect("Invoking `view` should always succeed");

    let (state, balance): (PiggyBankState, Amount) =
        from_bytes(&invoke.return_value).expect("View should always return a valid result");
    assert_eq!(state, PiggyBankState::Smashed, "Piggy bank is not smashed");
    assert_eq!(balance, Amount::zero(), "Piggy bank has non-zero balance after being smashed");
    assert_eq!(
        update.account_transfers().collect::<Vec<_>>(),
        [(
            initialization.contract_address,
            Amount::zero(),
            ALICE
        )],
        "The piggy bank made incorrect transfers when smashed"
    );
}