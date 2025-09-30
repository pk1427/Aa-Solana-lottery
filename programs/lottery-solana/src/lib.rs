use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};

declare_id!("5SfZAZiAB4f4SbXyu6fF3JyvgBEfGzTaTxifFDAjyRV1");

#[program]
pub mod lottery_solana {
    use super::*;

    //lottery initialize
    pub fn initialize(
        ctx: Context<Initialize>,
        id: u64,
        entry_price: u64,
        goal_amount: u64,
        start_time: i64,
        end_time: i64,
    ) -> Result<()> {
        msg!("Creating lottery...");

        let lottery = &mut ctx.accounts.lottery_account;

        //account fields
        lottery.authority = ctx.accounts.authority.key();
        lottery.treasury = ctx.accounts.treasury.key();
        lottery.id = id;
        lottery.entry_price = entry_price;
        lottery.goal_amount = goal_amount;
        lottery.total_raised = 0;
        lottery.start_time = start_time;
        lottery.end_time = end_time;
        lottery.state = LotteryState::Active;
        lottery.winner = None;
        lottery.participants = Vec::new();

        //registry
        let registry = &mut ctx.accounts.registry;
        registry.creator = ctx.accounts.authority.key();
        registry.lotteries.push(lottery.key());
        registry.active_lotteries.push(lottery.key()); //push to active lotteries

        Ok(())
    }

    //function to get active lotteries
    pub fn get_active_lotteries(ctx: Context<GetActiveLotteries>) -> Result<Vec<Pubkey>> {
        let registry = &ctx.accounts.registry;

        if registry.active_lotteries.is_empty() {
            msg!("No active lotteries found.");
            return Err(LotteryError::LotteryNotActive.into());
        }

        msg!("Found {} active lotteries", registry.active_lotteries.len());
        Ok(registry.active_lotteries.clone())
    }
    
    //function to buy lottery
    pub fn buy_lottery(ctx: Context<BuyLottery>) -> Result<()> {
        let lottery = &mut ctx.accounts.lottery_account;
        let buyer = &ctx.accounts.buyer;

        //trasfer funds to lottery pda
        let ix = system_instruction::transfer(
            &buyer.key(),
            &lottery.key(), 
            lottery.entry_price,
        );

        invoke(
            &ix,
            &[
                buyer.to_account_info(),
                lottery.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        //add participants
        if lottery.participants.contains(&buyer.key()) {
            msg!(
                "Buyer {} already joined lottery {}",
                buyer.key(),
                lottery.id
            );
            return Ok(());
        }

        lottery.participants.push(buyer.key());

        //total fund increase
        lottery.total_raised = lottery
            .total_raised
            .checked_add(lottery.entry_price)
            .ok_or(LotteryError::Overflow)?;

        //update the lottery state
        if lottery.total_raised >= lottery.goal_amount && lottery.state == LotteryState::Active {
            lottery.state = LotteryState::GoalMet;
            msg!(
                "Lottery {} goal reached! Total raised: {}",
                lottery.id,
                lottery.total_raised
            );
        } else {
            msg!(
                "Buyer {} joined lottery {}. Total raised: {}",
                buyer.key(),
                lottery.id,
                lottery.total_raised
            );
        }

        Ok(())
    }

    //function for pick winner
    // pub fn pick_winner(ctx: Context<PickWinner>) -> Result<()> {
    //     let lottery = &mut ctx.accounts.lottery_account;

    //     //only creator can call
    //     require!(
    //         lottery.authority == ctx.accounts.authority.key(),
    //         LotteryError::Unauthorized
    //     );

    //     //state must be GoalMet
    //     require!(
    //         lottery.state == LotteryState::GoalMet,
    //         LotteryError::LotteryNotActive
    //     );
    //     require!(lottery.winner.is_none(), LotteryError::AlreadyCompleted);

    //     // random winner 
    //     let participant_count = lottery.participants.len();
    //     require!(participant_count > 0, LotteryError::NoWinner);

    //     let winner_index = (participant_count + 7) % participant_count;
    //     let winner_pubkey = lottery.participants[winner_index];

    //     lottery.winner = Some(winner_pubkey);
    //     lottery.state = LotteryState::Completed;

    //     msg!(
    //         "Winner picked by creator {}: {}",
    //         ctx.accounts.authority.key(),
    //         winner_pubkey
    //     );

    //     msg!(
    //         "Lottery {} completed. Total funds remain in PDA: {} lamports",
    //         lottery.id,
    //         lottery.total_raised
    //     );

    //     Ok(())
    // }

    pub fn pick_winner(ctx: Context<PickWinner>) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery_account;

    // Only lottery creator can pick winner
    require!(
        lottery.authority == ctx.accounts.authority.key(),
        LotteryError::Unauthorized
    );

    // Lottery must have reached goal
    require!(
        lottery.state == LotteryState::GoalMet,
        LotteryError::LotteryNotActive
    );

    // Lottery should not already have a winner
    require!(
        lottery.winner.is_none(),
        LotteryError::AlreadyCompleted
    );

    let participant_count = lottery.participants.len();
    require!(participant_count > 0, LotteryError::NoWinner);

    // Parse randomness account
    let randomness_data = switchboard_on_demand::RandomnessAccountData::parse(
        ctx.accounts.randomness_account_data.data.borrow()
    ).map_err(|_| LotteryError::NoWinner)?;

    let clock = Clock::get()?;

    // Step 1: Commit phase
    if randomness_data.seed_slot != clock.slot {
        msg!("Committing randomness for future slot: {}", clock.slot + 1);
        // Here you would normally call a commit instruction from client
        // Store randomness account in lottery to reference it later
        lottery.randomness_account = Some(ctx.accounts.randomness_account_data.key());
        return Ok(());
    }

    // Step 2: Reveal phase
    let revealed_random_value = randomness_data
        .get_value(clock.slot)
        .map_err(|_| LotteryError::NoWinner)?;

    // Convert first 8 bytes of randomness to u64
    let random_bytes: [u8; 8] = revealed_random_value[0..8].try_into().unwrap();
    let random_number = u64::from_le_bytes(random_bytes);

    // Pick winner
    let winner_index = (random_number % participant_count as u64) as usize;
    let winner_pubkey = lottery.participants[winner_index];

    // Update lottery
    lottery.winner = Some(winner_pubkey);
    lottery.state = LotteryState::Completed;

    msg!("Winner picked randomly by creator {}: {}", ctx.accounts.authority.key(), winner_pubkey);
    msg!("Lottery {} completed. Total funds remain in PDA: {} lamports", lottery.id, lottery.total_raised);

    Ok(())
}



}

//main lottery account
#[account]
pub struct LotteryAccount {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub id: u64,
    pub entry_price: u64,
    pub goal_amount: u64,
    pub total_raised: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub state: LotteryState,
    pub winner: Option<Pubkey>,
    pub participants: Vec<Pubkey>,

    pub randomness_account: Option<Pubkey>,
}

//lottery states
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum LotteryState {
    Active,
    GoalMet,
    Completed,
    FailedRefunded,
}

//registry account to store lotteries
#[account]
pub struct LotteryRegistry {
    pub creator: Pubkey,
    pub lotteries: Vec<Pubkey>,        //stores all lotteries
    pub active_lotteries: Vec<Pubkey>, //active lotteries
}

//Error codes
#[error_code]
pub enum LotteryError {
    #[msg("Lottery is not active.")]
    LotteryNotActive,
    #[msg("Lottery has already ended.")]
    LotteryEnded,
    #[msg("Math overflow occurred.")]
    Overflow,
    #[msg("Lottery is full.")]
    LotteryFull,
    #[msg("Lottery already completed.")]
    AlreadyCompleted,
    #[msg("No winner available.")]
    NoWinner,
    #[msg("Only the creator can perform this action.")]
    Unauthorized,
}

//account linialization for creating lottery
#[derive(Accounts)]
#[instruction(id: u64)]
pub struct Initialize<'info> {
    #[account(
        init,
        space = 8 + 1024,
        payer = authority,
        seeds = [b"lottery", authority.key().as_ref(), &id.to_le_bytes()],
        bump
    )]
    pub lottery_account: Account<'info, LotteryAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This account is used as a treasury destination for funds.
    /// It is not required to be a specific program-derived address,
    /// so we skip Anchor's account validation here.
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,


    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + 1024,
        seeds = [b"registry", authority.key().as_ref()],
        bump
    )]
    pub registry: Account<'info, LotteryRegistry>,

    pub system_program: Program<'info, System>,
}

//for fetching active lotteries
#[derive(Accounts)]
pub struct GetActiveLotteries<'info> {
    #[account()]
    pub registry: Account<'info, LotteryRegistry>,
}

//for buy lotteries
#[derive(Accounts)]
pub struct BuyLottery<'info> {
    #[account(mut)]
    pub lottery_account: Account<'info, LotteryAccount>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

//for pick winner
#[derive(Accounts)]
pub struct PickWinner<'info> {
    #[account(
        mut,
        has_one = authority @ LotteryError::Unauthorized
    )]
    pub lottery_account: Account<'info, LotteryAccount>,

     /// CHECK: randomness account from Switchboard
    #[account(mut)]
    pub randomness_account_data: UncheckedAccount<'info>,

    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
