use anchor_lang::prelude::*;
use solana_program::{
    account_info::AccountInfo,
    system_instruction
};
use solana_program::program::invoke;
use anchor_spl::token::{self, CloseAccount, SetAuthority, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;


declare_id!("55KU278G7K2WqZaSfSecf7MBhCb76ckEbkrerwu9q94L");

#[program]
pub mod anchor_auction {
    use std::ops::Add;
    use super::*;
 
    const ESCROW_PDA_SEED: &[u8] = b"escrow";

    pub fn exhibit(
        ctx: Context<Exhibit>,
        initial_price: u64,
        auction_duration_sec: u32,
        sell_nft : u64,
    ) -> Result<()> {
        ctx.accounts.escrow_account.exhibitor_pubkey = ctx.accounts.exhibitor.key();
        ctx.accounts.escrow_account.exhibiting_nft_temp_pubkey = ctx.accounts.exhibitor_nft_temp_account.key();
        ctx.accounts.escrow_account.highest_bidder_pubkey = ctx.accounts.exhibitor.key();
        ctx.accounts.escrow_account.price = initial_price;
        ctx.accounts.escrow_account.end_at = ctx.accounts.clock.unix_timestamp.add(auction_duration_sec as i64);
        ctx.accounts.escrow_account.sell_price=sell_nft;
        
        let (pda, _bump_seed) = Pubkey::find_program_address(&[ESCROW_PDA_SEED], ctx.program_id);
        token::set_authority(
            ctx.accounts.to_set_authority_context(),
        AuthorityType::AccountOwner,                   
        Some(pda)
        )?;

        token::transfer(
            ctx.accounts.to_transfer_to_pda_context(),   
           1                                             
        )?;

        Ok(())
    }

    pub fn cancel(ctx: Context<Cancel> ) -> Result<()> {
        let (_, bump_seed) = Pubkey::find_program_address(&[ESCROW_PDA_SEED], ctx.program_id);
        let signers_seeds: &[&[&[u8]]] = &[&[&ESCROW_PDA_SEED[..], &[bump_seed]]];

        token::transfer(
            ctx.accounts
                .to_transfer_to_exhibitor_context()
                .with_signer(signers_seeds),
            ctx.accounts.exhibitor_nft_temp_account.amount
        )?;

        token::close_account(
            ctx.accounts
                .to_close_context()
                .with_signer(signers_seeds)
        )?;

        Ok(())
    }

    pub fn bid(ctx: Context<Bid>, price: u64) -> Result<()> {

        ctx.accounts.escrow_account.price = price;
        ctx.accounts.escrow_account.highest_bidder_pubkey = ctx.accounts.bidder.key();

        Ok(())
    }

    pub fn buynft(ctx: Context<Buynft>, buynft: u64) -> Result<()> {

        ctx.accounts.escrow_account.sell_price = buynft;
        ctx.accounts.escrow_account.highest_bidder_pubkey = ctx.accounts.bidder.key();

        Ok(())
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        let (_, bump_seed) = Pubkey::find_program_address(&[ESCROW_PDA_SEED], ctx.program_id);
        let signers_seeds: &[&[&[u8]]] = &[&[&ESCROW_PDA_SEED[..], &[bump_seed]]];

       // Transfer SOL from winning_bidder to exhibitor
       let sol_transfer_ix = system_instruction::transfer(
        &ctx.accounts.winning_bidder.key(),
          &ctx.accounts.exhibitor.key(),
            ctx.accounts.escrow_account.price,
        );

        // Invoke the SOL transfer instruction
        invoke(
        &sol_transfer_ix,
        &[
                   ctx.accounts.winning_bidder.to_account_info(),
                   ctx.accounts.exhibitor.to_account_info(),
                   ctx.accounts.system_program.to_account_info(),
                ],
        )?;

        token::transfer(
            ctx.accounts
                .to_transfer_to_highest_bidder_context()
                .with_signer(signers_seeds),
            ctx.accounts.exhibitor_nft_temp_account.amount,
        )?;

        token::close_account(
            ctx.accounts.to_close_nft_context()
                .with_signer(signers_seeds),
        )?;

        Ok(())
    }

    pub fn closenft(ctx: Context<Closenft>) -> Result<()> {
        let (_, bump_seed) = Pubkey::find_program_address(&[ESCROW_PDA_SEED], ctx.program_id);
        let signers_seeds: &[&[&[u8]]] = &[&[&ESCROW_PDA_SEED[..], &[bump_seed]]];

        // Transfer SOL from winning_bidder to exhibitor
       let sol_transfer_ix = system_instruction::transfer(
        &ctx.accounts.winning_bidder.key(),
          &ctx.accounts.exhibitor.key(),
            ctx.accounts.escrow_account.sell_price,
        );

        // Invoke the SOL transfer instruction
        invoke(
        &sol_transfer_ix,
        &[
                   ctx.accounts.winning_bidder.to_account_info(),
                   ctx.accounts.exhibitor.to_account_info(),
                   ctx.accounts.system_program.to_account_info(),
                ],
        )?;

        token::transfer(
            ctx.accounts
                .to_transfer_to_highest_bidder_context()
                .with_signer(signers_seeds),
            ctx.accounts.exhibitor_nft_temp_account.amount,
        )?;

        token::close_account(
            ctx.accounts.to_close_nft_context()
                .with_signer(signers_seeds),
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(initial_price: u64, auction_duration_sec: u32, sell_nft: u64)]
pub struct Exhibit<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(signer)]
    pub exhibitor: AccountInfo<'info>,
    #[account(
        mut,
        constraint = exhibitor_nft_token_account.amount == 1
    )]
    pub exhibitor_nft_token_account: Account<'info, TokenAccount>,
    pub exhibitor_nft_temp_account: Account<'info, TokenAccount>,
    #[account(zero)]
    pub escrow_account: Box<Account<'info, Auction>>,
    pub clock: Sysvar<'info, Clock>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Cancel<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(signer)]
    pub exhibitor: AccountInfo<'info>,
    #[account(mut)]
    pub exhibitor_nft_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub exhibitor_nft_temp_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = escrow_account.exhibitor_pubkey == exhibitor.key(),
        constraint = escrow_account.highest_bidder_pubkey == exhibitor.key(),
        constraint = escrow_account.exhibiting_nft_temp_pubkey == exhibitor_nft_temp_account.key(),
        close = exhibitor
    )]
    pub escrow_account: Box<Account<'info, Auction>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub pda: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(price: u64)]
pub struct Bid<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(signer, mut, constraint = bidder.lamports() >= price)]
    pub bidder: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = highest_bidder.key() != bidder.key()
    )]
    pub highest_bidder: AccountInfo<'info>, 
    #[account(
        mut,
        constraint = escrow_account.highest_bidder_pubkey == highest_bidder.key(),
        constraint = escrow_account.price < price,
        constraint = escrow_account.end_at > clock.unix_timestamp
    )]
    pub escrow_account: Box<Account<'info, Auction>>,
    pub clock: Sysvar<'info, Clock>,
    /// CHECK: This is not dangerous because we don't read or write from this account TODO check pda key
    pub pda: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(sell_price: u64)]
pub struct Buynft<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(signer, mut, constraint = bidder.lamports() >= sell_price)]
    pub bidder: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = highest_bidder.key() != bidder.key()
    )]
    pub highest_bidder: AccountInfo<'info>,
    #[account(
        mut,
        constraint = escrow_account.highest_bidder_pubkey == highest_bidder.key(),
        constraint = escrow_account.sell_price == sell_price,
        constraint = escrow_account.end_at > clock.unix_timestamp
    )]
    pub escrow_account: Box<Account<'info, Auction>>,
    pub clock: Sysvar<'info, Clock>,
    /// CHECK: This is not dangerous because we don't read or write from this account TODO check pda key
    pub pda: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(signer)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub winning_bidder: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub exhibitor: AccountInfo<'info>,
    #[account(mut)]
    pub exhibitor_nft_temp_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub highest_bidder_nft_receiving_account: Account<'info, TokenAccount>,
    #[account(
        mut, 
        constraint = escrow_account.exhibitor_pubkey == exhibitor.key(),
        constraint = escrow_account.exhibiting_nft_temp_pubkey == exhibitor_nft_temp_account.key(),
        constraint = escrow_account.highest_bidder_pubkey == winning_bidder.key(),
        constraint = escrow_account.end_at <= clock.unix_timestamp,
        close = exhibitor
    )]
    pub escrow_account: Box<Account<'info, Auction>>,
    pub clock: Sysvar<'info, Clock>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub pda: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Closenft<'info> {
    #[account(signer)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub winning_bidder: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub exhibitor: AccountInfo<'info>,
    #[account(mut)]
    pub exhibitor_nft_temp_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub highest_bidder_nft_receiving_account: Account<'info, TokenAccount>,
    #[account(
        mut, 
        constraint = escrow_account.exhibitor_pubkey == exhibitor.key(),
        constraint = escrow_account.exhibiting_nft_temp_pubkey == exhibitor_nft_temp_account.key(),
        constraint = escrow_account.highest_bidder_pubkey == winning_bidder.key(),
        close = exhibitor
    )]
    pub escrow_account: Box<Account<'info, Auction>>,
    pub clock: Sysvar<'info, Clock>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub pda: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    system_program: Program<'info, System>,
}

impl<'info> Exhibit<'info> {
    fn to_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self
                .exhibitor_nft_token_account
                .to_account_info()
                .clone(),
            to: self.exhibitor_nft_temp_account.to_account_info().clone(),
            authority: self.exhibitor.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn to_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.exhibitor_nft_temp_account.to_account_info().clone(),
            current_authority: self.exhibitor.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}


impl<'info> Cancel<'info> {
    fn to_transfer_to_exhibitor_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.exhibitor_nft_temp_account.to_account_info().clone(),
            to: self
                .exhibitor_nft_token_account
                .to_account_info()
                .clone(),
            authority: self.pda.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn to_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.exhibitor_nft_temp_account.to_account_info().clone(),
            destination: self.exhibitor.clone(),
            authority: self.pda.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}


impl<'info> Close<'info> {

    fn to_transfer_to_highest_bidder_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.exhibitor_nft_temp_account.to_account_info().clone(),
            to: self
                .highest_bidder_nft_receiving_account
                .to_account_info()
                .clone(),
            authority: self.pda.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn to_close_nft_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.exhibitor_nft_temp_account.to_account_info().clone(),
            destination: self.exhibitor.clone(),
            authority: self.pda.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

impl<'info> Closenft<'info> {

    fn to_transfer_to_highest_bidder_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.exhibitor_nft_temp_account.to_account_info().clone(),
            to: self
                .highest_bidder_nft_receiving_account
                .to_account_info()
                .clone(),
            authority: self.pda.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn to_close_nft_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.exhibitor_nft_temp_account.to_account_info().clone(),
            destination: self.exhibitor.clone(),
            authority: self.pda.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

/// see https://github.com/yoshidan/solana-auction/blob/main/program/src/state.rs#L10
#[account]
pub struct Auction {
    pub exhibitor_pubkey: Pubkey,
    pub exhibiting_nft_temp_pubkey: Pubkey,
    pub price: u64,
    pub end_at: i64,
    pub highest_bidder_pubkey: Pubkey,
    pub sell_price : u64,
}

