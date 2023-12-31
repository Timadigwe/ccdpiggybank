#![cfg_attr(not(feature = "std"), no_std)]

//! # A Concordium V1 smart contract
use concordium_std::*;
use core::fmt::Debug;




#[derive(Serialize, PartialEq, Eq, Debug, Clone, Copy)]
 pub enum PiggyBankState {
    Intact, 
    Smashed
}

#[derive(Debug, PartialEq, Eq, Serialize, Reject)]
pub enum SmashError {
    NotOwner,
    AlreadySmashed,
    TransferError, // Should never occur, see details below.
}

#[init(contract = "ccdpiggybank")]
fn init(_ctx: &InitContext, _state: &mut StateBuilder) -> InitResult<PiggyBankState> {
    Ok(PiggyBankState::Intact)
}

#[receive(contract = "ccdpiggybank", name = "insert", payable)]
fn piggy_insert(_ctx: &ReceiveContext, host: &Host<PiggyBankState>, _amount: Amount) -> ReceiveResult<()>{
        ensure!(*host.state() == PiggyBankState::Intact );
        Ok(())
}


#[receive(contract = "ccdpiggybank", name = "smash", mutable)]
fn piggy_smash(ctx: &ReceiveContext, host: &mut Host<PiggyBankState>) -> Result<(), SmashError> {
    let owner = ctx.owner();
    let sender = ctx.sender();
    ensure!(sender.matches_account(&owner), SmashError::NotOwner);
    ensure!(*host.state() == PiggyBankState::Intact, SmashError::AlreadySmashed);

    *host.state_mut() = PiggyBankState::Smashed;

    let balance = host.self_balance();
    let transfer_result = host.invoke_transfer(&owner, balance);
    ensure!(transfer_result.is_ok(), SmashError::TransferError);
    Ok(())
}

#[receive(contract = "ccdpiggybank", name = "view")]
fn piggy_view(_ctx: &ReceiveContext, host: &Host<PiggyBankState>) -> ReceiveResult<(Amount, PiggyBankState)> {
    let current_state = *host.state();
    let current_balance = host.self_balance();
    Ok((current_balance, current_state))
}