use anchor_lang::prelude::*;
// use solana_program::keccak::hashv;

// use crate::constants::MAX_LEVEL;
use crate::error::Error;

// pub fn calc_level(key: &[u8]) -> u8 {
//     // 确定性 level：用 keccak hash
//     // let h = hashv(&[key]).0;
//     let h: [u8; 32] = hashv(&[key]).to_bytes();
//     let mut lvl = 1u8;
//     for i in 0..MAX_LEVEL {
//         if (h[i] & 1) == 1 {
//             lvl += 1;
//         } else {
//             break;
//         }
//     }
//     lvl.min(MAX_LEVEL as u8)
// }

// pub fn key_cmp(a: &[u8], b: &[u8]) -> core::cmp::Ordering {
//     a.cmp(b)
// }

// pub fn charge(payer: &Signer, treasury: &AccountInfo, amount: u64) -> Result<()> {
//     require!(
//         **payer.to_account_info().lamports.borrow() >= amount,
//         Error::InsufficientFee
//     );

//     **payer.to_account_info().try_borrow_mut_lamports()? -= amount;
//     **treasury.try_borrow_mut_lamports()? += amount;

//     Ok(())
// }

// 增加 system_program 参数
pub fn charge<'info>(
    payer: &Signer<'info>,
    treasury: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>, // 🌟 新增这个参数
    amount: u64,
) -> Result<()> {
    require!(
        **payer.to_account_info().lamports.borrow() >= amount,
        Error::InsufficientFee
    );

    // 🌟 使用传入的 system_program 账户
    let cpi_context = CpiContext::new(
        system_program.clone(),
        anchor_lang::system_program::Transfer {
            from: payer.to_account_info(),
            to: treasury.clone(),
        },
    );

    anchor_lang::system_program::transfer(cpi_context, amount)?;

    Ok(())
}
