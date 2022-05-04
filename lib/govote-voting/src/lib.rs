//! This is the first voting contract.
//! The way to create a voting agenda is to initialize the contract with the title, description and proposals as parameters.
//! The function that gives voting rights is excluded in this version.
//! The code related to the ability to grant voting rights is commented out.
//!
//! The current specifications are as follows.
//! - You can vote with any account.
//! - Each account has one vote.
//! - You can change the options until the voting is completed.
//!
//! **WARNING** In this version you can do the following for testing:
//! - Even after the deadline has passed, you can vote if the data is not counted.
//! - Anyone can execute the aggregation method.
//! - Aggregation is possible even before the deadline.

use concordium_std::{collections::HashMap as Map, *};

type ProposalId = u8;
type ProposalNames = Vec<String>;
type Title = String;
type Description = String;

#[derive(Debug, Serialize, SchemaType, Default, PartialEq)]
struct VoterState {
    weight: u32,
    voted: bool,
    vote: ProposalId,
}

#[derive(Debug, Serialize, SchemaType, Default, PartialEq)]
struct Proposal {
    name: String,
    vote_count: u32,
}

#[derive(Serialize, SchemaType)]
struct InitParams {
    title: Title,
    description: Description,
    proposal_names: ProposalNames,
    expiry: Timestamp,
}

impl Proposal {
    fn new(name: String) -> Self {
        Proposal {
            name,
            vote_count: 0,
        }
    }
}

#[derive(Serialize, SchemaType)]
struct GetVoterParams {
    voter_address: Address,
}

#[derive(Serialize, SchemaType)]
struct GetVoteParams {
    proposal_id: ProposalId,
}

/// Contract error type
#[derive(Debug, PartialEq, Eq, Reject)]
enum ContractError {
    /// Failed parsing the parameter.
    #[from(ParseError)]
    ParseParams,
    /// Failed logging: Log is full.
    LogFull,
    /// Failed logging: Log is malformed.
    LogMalformed,
    /// The transfer is not from the owner of the vote.
    // FromIsNotTheOwner,
    /// The voter already voted.
    // AlreadyVoted,
    /// The voter already has right to vote.
    // AlreadyHasRightToVote,
    /// The voter doesn't have right to vote.
    // NoRightToVote,
    /// Already finished.
    AlreadyFinished,
    /// exipred for voting.
    // Expired,
    /// not exipred for tallying.
    // NotExpired,
    /// Voter is not found.
    VoterIsNotFound,
    /// Voter did not vote.
    NotVoted,
    /// Proposal is not found.
    ProposalIsNotFound,
}

// [TODO]: ロギング用のイベントの定義をする。
/// Event to be printed in the log.
#[derive(Serialize)]
enum Event {
    GiveRightToVote {
        to: Address,
        added_weight: u32,
        total_weight: u32,
    },
}

type ContractResult<A> = Result<A, ContractError>;

impl From<LogError> for ContractError {
    fn from(le: LogError) -> Self {
        match le {
            LogError::Full => Self::LogFull,
            LogError::Malformed => Self::LogMalformed,
        }
    }
}

#[derive(Debug, Serialize, SchemaType, PartialEq)]
enum Status {
    InProcess,
    Finished,
}

#[contract_state(contract = "govote_voting")]
#[derive(Debug, Serialize, SchemaType, PartialEq)]
struct State {
    voters: Map<Address, VoterState>,
    proposals: Map<ProposalId, Proposal>,
    status: Status,
    winning_proposal_id: Vec<ProposalId>,
    title: Title,
    description: Description,
    expiry: Timestamp,
}

impl State {
    fn new(
        title: Title,
        description: Description,
        proposal_names: ProposalNames,
        expiry: Timestamp,
    ) -> Self {
        let mut proposals = Map::default();
        for (i, proposal_name) in proposal_names.iter().enumerate() {
            proposals.insert(i as ProposalId, Proposal::new(proposal_name.to_string()));
        }

        State {
            voters: Map::default(),
            proposals,
            status: Status::InProcess,
            winning_proposal_id: vec![],
            title,
            description,
            expiry,
        }
    }

    /// Get the approve of a token.
    fn get_voter(&self, voter_address: &Address) -> Option<&VoterState> {
        self.voters.get(voter_address)
    }

    fn add_vote_count(&mut self, proposal_id: &ProposalId, weight: u32) {
        let proposal = self.proposals.entry(*proposal_id).or_insert_with(Proposal::default);
        proposal.vote_count += weight;
    }

    fn subtract_vote_count(&mut self, proposal_id: &ProposalId, weight: u32) {
        let proposal = self.proposals.entry(*proposal_id).or_insert_with(Proposal::default);
        proposal.vote_count -= weight;
    }
}

/// Init function that creates a new contract.
#[init(contract = "govote_voting", parameter = "InitParams")]
fn contract_init(ctx: &impl HasInitContext) -> InitResult<State> {
    let params: InitParams = ctx.parameter_cursor().get()?;
    let state = State::new(params.title, params.description, params.proposal_names, params.expiry);
    Ok(state)
}

/// Add right to vote.
/// Only be called by owner.
// #[receive(contract = "govote_voting", name = "giveRightToVote", parameter = "GetVoterParams")]
// fn contract_give_right_to_vote<A: HasActions>(
//     ctx: &impl HasReceiveContext,
//     state: &mut State,
// ) -> ContractResult<A> {
//     let params: GetVoterParams = ctx.parameter_cursor().get()?;
//     let owner = ctx.owner();
//     let sender = ctx.sender();

//     // 集計が終わってなければ実行できる。
//     ensure!(state.status != Status::Finished, ContractError::AlreadyFinished);

//     // expiryを超えていなければ実行できる。
//     let slot_time = ctx.metadata().slot_time();
//     ensure!(slot_time <= state.expiry, ContractError::Expired);

//     // ownerだけが実行できる。
//     ensure!(sender.matches_account(&owner), ContractError::FromIsNotTheOwner);

//     // votersをアドレスから取得。なければvotersに新規追加。
//     let voter_state =
//         &mut state.voters.entry(params.voter_address).or_insert_with(VoterState::default);

//     // 投票済みならエラー。
//     ensure!(!voter_state.voted, ContractError::AlreadyVoted);

//     // weightが0以上でエラー。
//     ensure!(voter_state.weight == 0, ContractError::AlreadyHasRightToVote);

//     voter_state.weight = 1;

//     // [TODO]: AddWeightログを追加

//     Ok(A::accept())
// }

/// Vote to proposal.
#[receive(contract = "govote_voting", name = "vote", parameter = "GetVoteParams")]
fn contract_vote<A: HasActions>(
    ctx: &impl HasReceiveContext,
    state: &mut State,
) -> ContractResult<A> {
    let params: GetVoteParams = ctx.parameter_cursor().get()?;
    let sender_address = ctx.sender();

    // proposalが存在すれば実行できる。
    state.proposals.get_mut(&params.proposal_id).ok_or(ContractError::ProposalIsNotFound)?;

    // 集計が終わってなければ実行できる。
    ensure!(state.status != Status::Finished, ContractError::AlreadyFinished);

    // expiryを超えていなければ実行できる。
    // let slot_time = ctx.metadata().slot_time();
    // ensure!(slot_time <= state.expiry, ContractError::Expired);

    if state.get_voter(&sender_address) != None {
        // 投票済みならweight分のvote_countを引く
        if state.get_voter(&sender_address).map(|a| a.voted) == Some(true) {
            state.subtract_vote_count(
                &state.get_voter(&sender_address).map(|a| a.vote).unwrap(),
                state.get_voter(&sender_address).map(|a| a.weight).unwrap(),
            );
        }
        // ensure!(
        //     state.get_voter(&sender_address).map(|a| a.voted) == Some(false),
        //     ContractError::AlreadyVoted
        // );
    }

    // ensure!(
    //     state.get_voter(&sender_address).map(|a| a.weight) != Some(0),
    //     ContractError::NoRightToVote
    // );

    let voter_state = state.voters.entry(sender_address).or_insert_with(VoterState::default);
    voter_state.voted = true;
    voter_state.weight = 1;
    voter_state.vote = params.proposal_id;

    state.add_vote_count(
        &params.proposal_id,
        state.get_voter(&sender_address).map(|a| a.weight).unwrap(),
    );

    Ok(A::accept())
}

/// 集計
#[receive(contract = "govote_voting", name = "winningProposal")]
fn contract_winning_proposal<A: HasActions>(
    _ctx: &impl HasReceiveContext,
    state: &mut State,
) -> ContractResult<A> {
    let mut winning_vote_count = 0;
    let mut winning_proposal_id = vec![];

    // 集計が終わってなければ実行できる。
    ensure!(state.status != Status::Finished, ContractError::AlreadyFinished);

    // expiryを超えていれば実行できる。
    // let slot_time = ctx.metadata().slot_time();
    // ensure!(state.expiry < slot_time, ContractError::NotExpired);

    for (proposal_id, proposal) in state.proposals.iter() {
        if winning_vote_count < proposal.vote_count {
            winning_vote_count = proposal.vote_count;
            winning_proposal_id = [*proposal_id].to_vec();
        } else if winning_vote_count == proposal.vote_count {
            winning_proposal_id.push(*proposal_id)
        }
    }

    state.status = Status::Finished;
    state.winning_proposal_id = winning_proposal_id;

    Ok(A::accept())
}

/// 投票のキャンセル
#[receive(contract = "govote_voting", name = "cancelVote")]
fn cancel_vote<A: HasActions>(
    ctx: &impl HasReceiveContext,
    state: &mut State,
) -> ContractResult<A> {
    let sender_address = ctx.sender();

    // 集計が終わってなければ実行できる。
    ensure!(state.status != Status::Finished, ContractError::AlreadyFinished);

    // expiryを超えていなければ実行できる。
    // let slot_time = ctx.metadata().slot_time();
    // ensure!(slot_time <= state.expiry, ContractError::Expired);

    let mut voter = state.voters.get_mut(&sender_address).ok_or(ContractError::VoterIsNotFound)?;
    ensure!(voter.voted == true, ContractError::NotVoted);

    let proposal = state.proposals.get_mut(&voter.vote).ok_or(ContractError::ProposalIsNotFound)?;
    proposal.vote_count -= voter.weight;

    voter.voted = false;
    voter.vote = 0;

    Ok(A::accept())
}

#[concordium_cfg_test]
mod tests {
    use super::*;
    use concordium_std::test_infrastructure::*;
    use std::sync::atomic::{AtomicU8, Ordering};

    static ADDRESS_COUNTER: AtomicU8 = AtomicU8::new(1);

    const ACCOUNT_0: AccountAddress = AccountAddress([0u8; 32]);
    // const ADDRESS_0: Address = Address::Account(ACCOUNT_0);
    const ACCOUNT_1: AccountAddress = AccountAddress([1u8; 32]);
    // const ADDRESS_1: Address = Address::Account(ACCOUNT_1);
    const TITLE: &str = "Test Title";
    const DESCRIPTION: &str = "This is test description.";
    const PROPOSAL_NAME_1: &str = "This is first test proposal.";
    const PROPOSAL_NAME_2: &str = "This is second test proposal.";
    const EXPIRY: u64 = 1;

    #[allow(unused)]
    fn new_account() -> AccountAddress {
        let account = AccountAddress([ADDRESS_COUNTER.load(Ordering::SeqCst); 32]);
        ADDRESS_COUNTER.fetch_add(1, Ordering::SeqCst);
        account
    }

    fn init_parameter() -> InitParams {
        let mut init_vec = Vec::new();
        init_vec.push(PROPOSAL_NAME_1.to_string());
        init_vec.push(PROPOSAL_NAME_2.to_string());

        InitParams {
            title: TITLE.to_string(),
            description: DESCRIPTION.to_string(),
            proposal_names: init_vec,
            expiry: Timestamp::from_timestamp_millis(EXPIRY),
        }
    }

    fn create_parameter_bytes(parameter: &InitParams) -> Vec<u8> {
        to_bytes(parameter)
    }

    fn parametrized_init_ctx<'a>(parameter_bytes: &'a Vec<u8>) -> InitContextTest<'a> {
        let mut ctx = InitContextTest::empty();
        ctx.set_parameter(parameter_bytes);
        ctx
    }

    fn receive_ctx<'a>(
        owner: AccountAddress,
        sender: AccountAddress,
        slot_time: u64,
        parameter_bytes: &'a Vec<u8>,
    ) -> ReceiveContextTest<'a> {
        let mut ctx = ReceiveContextTest::empty();
        ctx.set_sender(Address::Account(sender));
        ctx.set_owner(owner);
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(slot_time));
        ctx.set_parameter(parameter_bytes);
        ctx
    }

    #[concordium_test]
    fn test_init() {
        let mut init_vec = Vec::new();
        init_vec.push(PROPOSAL_NAME_1.to_string());
        init_vec.push(PROPOSAL_NAME_2.to_string());

        let parameter_bytes = create_parameter_bytes(&init_parameter());
        let ctx = parametrized_init_ctx(&parameter_bytes);

        let state_result = contract_init(&ctx);
        let state = state_result.expect("Contract initialization results in error");

        claim_eq!(
            state,
            State::new(
                TITLE.to_string(),
                DESCRIPTION.to_string(),
                init_vec,
                Timestamp::from_timestamp_millis(EXPIRY)
            ),
            "State is not equal."
        );
    }

    // #[concordium_test]
    // fn test_give_right_to_vote() {
    //     let parameter_bytes = create_parameter_bytes(&init_parameter());
    //     let ctx = parametrized_init_ctx(&parameter_bytes);
    //     let state_result = contract_init(&ctx);
    //     let mut state = state_result.expect("Contract initialization results in error");

    //     let account1 = new_account();
    //     let params = GetVoterParams {
    //         voter_address: Address::Account(account1),
    //     };
    //     let parameter_bytes = to_bytes(&params);
    //     let slot_time = 0u64;
    //     let ctx = receive_ctx(ACCOUNT_0, ACCOUNT_0, slot_time, &parameter_bytes);
    //     let res: ContractResult<ActionsTree> = contract_give_right_to_vote(&ctx, &mut state);
    //     let actions = res.expect_report("giving right results in error.");
    //     claim_eq!(actions, ActionsTree::accept(), "No action should be produced.");

    //     let account2 = new_account();
    //     let params = GetVoterParams {
    //         voter_address: Address::Account(account2),
    //     };
    //     let parameter_bytes = to_bytes(&params);
    //     let slot_time = 0u64;
    //     let ctx = receive_ctx(ACCOUNT_0, ACCOUNT_0, slot_time, &parameter_bytes);
    //     let res: ContractResult<ActionsTree> = contract_give_right_to_vote(&ctx, &mut state);
    //     let actions = res.expect_report("giving right results in error.");
    //     claim_eq!(actions, ActionsTree::accept(), "No action should be produced.");

    //     let mut voters = Map::default();
    //     voters.insert(
    //         Address::Account(account1),
    //         VoterState {
    //             weight: 1,
    //             ..Default::default()
    //         },
    //     );
    //     voters.insert(
    //         Address::Account(account2),
    //         VoterState {
    //             weight: 1,
    //             ..Default::default()
    //         },
    //     );
    //     claim_eq!(state.voters, voters);
    // }

    // #[concordium_test]
    // fn test_give_right_to_vote_expired() {
    //     let parameter_bytes = create_parameter_bytes(&init_parameter());
    //     let ctx = parametrized_init_ctx(&parameter_bytes);
    //     let state_result = contract_init(&ctx);
    //     let mut state = state_result.expect("Contract initialization results in error");

    //     let account1 = new_account();
    //     let params = GetVoterParams {
    //         voter_address: Address::Account(account1),
    //     };
    //     let parameter_bytes = to_bytes(&params);
    //     let slot_time = 10u64;
    //     let ctx = receive_ctx(ACCOUNT_0, ACCOUNT_0, slot_time, &parameter_bytes);
    //     let res: ContractResult<ActionsTree> = contract_give_right_to_vote(&ctx, &mut state);
    //     let err = res.expect_err_report("Contract is expected to fail.");
    //     claim_eq!(err, ContractError::Expired, "Expected to fail with error Expired");
    // }

    // #[concordium_test]
    // fn test_give_right_to_vote_with_no_authority() {
    //     let parameter_bytes = create_parameter_bytes(&init_parameter());
    //     let ctx = parametrized_init_ctx(&parameter_bytes);
    //     let state_result = contract_init(&ctx);
    //     let mut state = state_result.expect("Contract initialization results in error");

    //     let account1 = new_account();
    //     let params = GetVoterParams {
    //         voter_address: Address::Account(account1),
    //     };
    //     let parameter_bytes = to_bytes(&params);
    //     let slot_time = 0u64;
    //     let ctx = receive_ctx(ACCOUNT_0, account1, slot_time, &parameter_bytes);
    //     let res: ContractResult<ActionsTree> = contract_give_right_to_vote(&ctx, &mut state);
    //     let err = res.expect_err_report("Contract is expected to fail.");
    //     claim_eq!(
    //         err,
    //         ContractError::FromIsNotTheOwner,
    //         "Expected to fail with error FromIsNotTheOwner"
    //     );
    // }

    #[concordium_test]
    fn test_contract_vote() {
        let parameter_bytes = create_parameter_bytes(&init_parameter());
        let ctx = parametrized_init_ctx(&parameter_bytes);
        let state_result = contract_init(&ctx);
        let mut state = state_result.expect("Contract initialization results in error");

        let account1 = new_account();
        let params = GetVoteParams {
            proposal_id: 1 as ProposalId,
        };
        let parameter_bytes = to_bytes(&params);
        let slot_time = 0u64;
        let ctx = receive_ctx(ACCOUNT_0, account1, slot_time, &parameter_bytes);

        let res: ContractResult<ActionsTree> = contract_vote(&ctx, &mut state);
        let actions = res.expect_report("contract voting results in error.");
        claim_eq!(actions, ActionsTree::accept(), "No action should be produced.");

        claim_eq!(
            state.voters.get(&Address::Account(account1)).unwrap().voted,
            true,
            "voted status should be true"
        );

        claim_eq!(
            state.proposals.get(&1).unwrap().vote_count,
            1,
            "something wrong with vote_count"
        );
    }

    #[concordium_test]
    fn test_contract_vote_to_wrong_number() {
        let parameter_bytes = create_parameter_bytes(&init_parameter());
        let ctx = parametrized_init_ctx(&parameter_bytes);
        let state_result = contract_init(&ctx);
        let mut state = state_result.expect("Contract initialization results in error");

        let account1 = new_account();
        let params = GetVoteParams {
            proposal_id: 2 as ProposalId,
        };
        let parameter_bytes = to_bytes(&params);
        let slot_time = 0u64;
        let ctx = receive_ctx(ACCOUNT_0, account1, slot_time, &parameter_bytes);

        let res_1: Result<ActionsTree, ContractError> = contract_vote(&ctx, &mut state);
        claim_eq!(
            res_1,
            Err(ContractError::ProposalIsNotFound),
            "Result should be ProposalIsNotFound."
        );
    }

    #[concordium_test]
    fn test_contract_vote_selection_change() {
        let parameter_bytes = create_parameter_bytes(&init_parameter());
        let ctx = parametrized_init_ctx(&parameter_bytes);
        let state_result = contract_init(&ctx);
        let mut state = state_result.expect("Contract initialization results in error");

        let account1 = new_account();
        let params = GetVoteParams {
            proposal_id: 0 as ProposalId,
        };
        let parameter_bytes = to_bytes(&params);
        let slot_time = 0u64;
        let ctx = receive_ctx(ACCOUNT_0, account1, slot_time, &parameter_bytes);

        let res_1: ContractResult<ActionsTree> = contract_vote(&ctx, &mut state);
        let actions_1 = res_1.expect_report("contract voting results in error.");
        claim_eq!(actions_1, ActionsTree::accept(), "No action should be produced.");

        claim_eq!(
            state.proposals.get(&0).unwrap().vote_count,
            1,
            "something wrong with vote_count"
        );

        claim_eq!(
            state.proposals.get(&1).unwrap().vote_count,
            0,
            "something wrong with vote_count"
        );

        let params = GetVoteParams {
            proposal_id: 1 as ProposalId,
        };
        let parameter_bytes = to_bytes(&params);
        let slot_time = 0u64;
        let ctx = receive_ctx(ACCOUNT_0, account1, slot_time, &parameter_bytes);
        let res_2: ContractResult<ActionsTree> = contract_vote(&ctx, &mut state);
        let actions_2 = res_2.expect_report("contract voting results in error.");
        claim_eq!(actions_2, ActionsTree::accept(), "No action should be produced.");

        claim_eq!(
            state.voters.get(&Address::Account(account1)).unwrap().voted,
            true,
            "voted status should be true"
        );

        claim_eq!(
            state.proposals.get(&0).unwrap().vote_count,
            0,
            "something wrong with vote_count"
        );

        claim_eq!(
            state.proposals.get(&1).unwrap().vote_count,
            1,
            "something wrong with vote_count"
        );
    }

    #[concordium_test]
    fn test_contract_vote_same_selection() {
        let parameter_bytes = create_parameter_bytes(&init_parameter());
        let ctx = parametrized_init_ctx(&parameter_bytes);
        let state_result = contract_init(&ctx);
        let mut state = state_result.expect("Contract initialization results in error");

        let account1 = new_account();
        let params = GetVoteParams {
            proposal_id: 1 as ProposalId,
        };
        let parameter_bytes = to_bytes(&params);
        let slot_time = 0u64;
        let ctx = receive_ctx(ACCOUNT_0, account1, slot_time, &parameter_bytes);

        let res_1: ContractResult<ActionsTree> = contract_vote(&ctx, &mut state);
        let actions_1 = res_1.expect_report("contract voting results in error.");
        claim_eq!(actions_1, ActionsTree::accept(), "No action should be produced.");

        let slot_time = 0u64;
        let ctx = receive_ctx(ACCOUNT_0, account1, slot_time, &parameter_bytes);

        let res_2: ContractResult<ActionsTree> = contract_vote(&ctx, &mut state);
        let actions_2 = res_2.expect_report("contract voting results in error.");
        claim_eq!(actions_2, ActionsTree::accept(), "No action should be produced.");

        claim_eq!(
            state.voters.get(&Address::Account(account1)).unwrap().voted,
            true,
            "voted status should be true"
        );

        claim_eq!(
            state.proposals.get(&1).unwrap().vote_count,
            1,
            "something wrong with vote_count"
        );
    }

    #[concordium_test]
    fn test_contract_winning_proposal_no_voters() {
        let parameter_bytes = create_parameter_bytes(&init_parameter());
        let ctx = parametrized_init_ctx(&parameter_bytes);
        let state_result = contract_init(&ctx);
        let mut state = state_result.expect("Contract initialization results in error");

        let account1 = new_account();
        let params = GetVoteParams {
            proposal_id: 1 as ProposalId,
        };
        let parameter_bytes = to_bytes(&params);
        let slot_time = 0u64;
        let ctx = receive_ctx(ACCOUNT_0, account1, slot_time, &parameter_bytes);
        claim_eq!(state.status, Status::InProcess, "Status should be InProcess");
        let res_1: ContractResult<ActionsTree> = contract_winning_proposal(&ctx, &mut state);
        let actions_1 = res_1.expect_report("contract winning proposal results in error.");
        claim_eq!(actions_1, ActionsTree::accept(), "No action should be produced.");
        claim_eq!(state.status, Status::Finished, "Status should be Finished");
        claim_eq!(
            state.winning_proposal_id,
            vec![1, 0],
            "something wrong with winning_proposal_id"
        );
    }

    #[concordium_test]
    fn test_contract_winning_proposal_one_winning_proposal() {
        let parameter_bytes = create_parameter_bytes(&init_parameter());
        let ctx = parametrized_init_ctx(&parameter_bytes);
        let state_result = contract_init(&ctx);
        let mut state = state_result.expect("Contract initialization results in error");

        let account1 = new_account();
        let params = GetVoteParams {
            proposal_id: 0 as ProposalId,
        };
        let parameter_bytes = to_bytes(&params);
        let slot_time = 0u64;
        let ctx = receive_ctx(ACCOUNT_0, account1, slot_time, &parameter_bytes);

        let res_1: ContractResult<ActionsTree> = contract_vote(&ctx, &mut state);
        let actions_1 = res_1.expect_report("contract voting results in error.");
        claim_eq!(actions_1, ActionsTree::accept(), "No action should be produced.");

        claim_eq!(state.status, Status::InProcess, "Status should be InProcess");

        claim_eq!(
            state.voters.get(&Address::Account(account1)).unwrap().voted,
            true,
            "voted status should be true"
        );

        claim_eq!(
            state.proposals.get(&0).unwrap().vote_count,
            1,
            "something wrong with vote_count"
        );

        let res_2: ContractResult<ActionsTree> = contract_winning_proposal(&ctx, &mut state);
        let actions_2 = res_2.expect_report("contract winning proposal results in error.");
        claim_eq!(actions_2, ActionsTree::accept(), "No action should be produced.");
        claim_eq!(state.status, Status::Finished, "Status should be Finished");
        claim_eq!(state.winning_proposal_id, vec![0], "something wrong with winning_proposal_id");
    }

    #[concordium_test]
    fn test_contract_winning_proposal_multi_winning_proposal() {
        let parameter_bytes = create_parameter_bytes(&init_parameter());
        let ctx = parametrized_init_ctx(&parameter_bytes);
        let state_result = contract_init(&ctx);
        let mut state = state_result.expect("Contract initialization results in error");

        let account1 = new_account();
        let params = GetVoteParams {
            proposal_id: 0 as ProposalId,
        };
        let parameter_bytes = to_bytes(&params);
        let slot_time = 0u64;
        let ctx = receive_ctx(ACCOUNT_0, account1, slot_time, &parameter_bytes);

        let res_1: ContractResult<ActionsTree> = contract_vote(&ctx, &mut state);
        let actions_1 = res_1.expect_report("contract voting results in error.");
        claim_eq!(actions_1, ActionsTree::accept(), "No action should be produced.");

        claim_eq!(state.status, Status::InProcess, "Status should be InProcess");

        claim_eq!(
            state.voters.get(&Address::Account(account1)).unwrap().voted,
            true,
            "voted status should be true"
        );

        claim_eq!(
            state.proposals.get(&0).unwrap().vote_count,
            1,
            "something wrong with vote_count"
        );

        claim_eq!(
            state.proposals.get(&1).unwrap().vote_count,
            0,
            "something wrong with vote_count"
        );

        let account2 = new_account();
        let params = GetVoteParams {
            proposal_id: 1 as ProposalId,
        };
        let parameter_bytes = to_bytes(&params);
        let slot_time = 0u64;
        let ctx = receive_ctx(ACCOUNT_1, account2, slot_time, &parameter_bytes);

        let res_2: ContractResult<ActionsTree> = contract_vote(&ctx, &mut state);
        let actions_2 = res_2.expect_report("contract voting results in error.");
        claim_eq!(actions_2, ActionsTree::accept(), "No action should be produced.");

        claim_eq!(state.status, Status::InProcess, "Status should be InProcess");

        claim_eq!(
            state.voters.get(&Address::Account(account1)).unwrap().voted,
            true,
            "voted status should be true"
        );

        claim_eq!(
            state.proposals.get(&0).unwrap().vote_count,
            1,
            "something wrong with vote_count"
        );

        claim_eq!(
            state.proposals.get(&1).unwrap().vote_count,
            1,
            "something wrong with vote_count"
        );

        let res_3: ContractResult<ActionsTree> = contract_winning_proposal(&ctx, &mut state);
        let actions_3 = res_3.expect_report("contract winning proposal results in error.");

        claim_eq!(actions_3, ActionsTree::accept(), "No action should be produced.");
        claim_eq!(state.status, Status::Finished, "Status should be Finished");
        claim_eq!(
            state.winning_proposal_id,
            vec![1, 0],
            "something wrong with winning_proposal_id"
        );
    }

    #[concordium_test]
    fn test_cancel_vote() {
        let parameter_bytes = create_parameter_bytes(&init_parameter());
        let ctx = parametrized_init_ctx(&parameter_bytes);
        let state_result = contract_init(&ctx);
        let mut state = state_result.expect("Contract initialization results in error");

        let account1 = new_account();
        let params = GetVoteParams {
            proposal_id: 1 as ProposalId,
        };
        let parameter_bytes = to_bytes(&params);
        let slot_time = 0u64;
        let ctx = receive_ctx(ACCOUNT_0, account1, slot_time, &parameter_bytes);
        let res: ContractResult<ActionsTree> = contract_vote(&ctx, &mut state);
        let actions = res.expect_report("contract voting results in error.");
        claim_eq!(actions, ActionsTree::accept(), "No action should be produced.");
        claim_eq!(
            state.voters.get(&Address::Account(account1)).unwrap().voted,
            true,
            "voted status should be true"
        );
        claim_eq!(
            state.proposals.get(&1).unwrap().vote_count,
            1,
            "something wrong with vote_count"
        );

        let res: ContractResult<ActionsTree> = cancel_vote(&ctx, &mut state);
        let actions = res.expect_report("cancel voting results in error.");
        claim_eq!(actions, ActionsTree::accept(), "No action should be produced.");
        claim_eq!(
            state.voters.get(&Address::Account(account1)).unwrap().voted,
            false,
            "voted status should be true"
        );
        claim_eq!(
            state.proposals.get(&1).unwrap().vote_count,
            0,
            "something wrong with vote_count"
        );
    }
}
