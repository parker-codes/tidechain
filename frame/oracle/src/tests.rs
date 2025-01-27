// Copyright 2021-2022 Semantic Network Ltd.
// This file is part of Tidechain.

// Tidechain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tidechain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tidechain.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
  mock::{
    new_test_ext, AccountId, Adapter, Assets, Event as MockEvent, FeeAmount, Fees,
    MarketMakerFeeAmount, Oracle, Origin, Sunrise, System, Test,
  },
  pallet::*,
};
use frame_support::{
  assert_noop, assert_ok,
  traits::fungibles::{Inspect, InspectHold, Mutate},
};
use sp_core::H256;
use sp_runtime::{
  traits::{BadOrigin, Zero},
  FixedPointNumber, FixedU128, Permill,
};
use std::str::FromStr;
use tidefi_primitives::{
  pallet::{FeesExt, OracleExt},
  Balance, CurrencyId, Hash, SwapConfirmation, SwapStatus, SwapType,
};

const CURRENT_BLOCK_NUMBER: BlockNumber = 0;

// TEMP Asset
const TEMP_ASSET_ID: u32 = 4;
const TEMP_CURRENCY_ID: CurrencyId = CurrencyId::Wrapped(TEMP_ASSET_ID);
const TEMP_ASSET_IS_SUFFICIENT: bool = true;
const TEMP_ASSET_MIN_BALANCE: u128 = 1;

// TEMP Asset Metadata
const TEMP_ASSET_NAME: &str = "TEMP";
const TEMP_ASSET_SYMBOL: &str = "TEMP";
const TEMP_ASSET_NUMBER_OF_DECIMAL_PLACES: u8 = 8;

// TEMP2 Asset
const TEMP2_ASSET_ID: u32 = TEMP_ASSET_ID + 1;
const TEMP2_CURRENCY_ID: CurrencyId = CurrencyId::Wrapped(TEMP2_ASSET_ID);

// TEMP2 Asset Metadata
const TEMP2_ASSET_NAME: &str = "TEMP2";
const TEMP2_ASSET_SYMBOL: &str = "TEMP2";
const TEMP2_ASSET_NUMBER_OF_DECIMAL_PLACES: u8 = 2;

// ZEMP Asset
const ZEMP_ASSET_ID: u32 = 5;
const ZEMP_CURRENCY_ID: CurrencyId = CurrencyId::Wrapped(ZEMP_ASSET_ID);
const ZEMP_ASSET_IS_SUFFICIENT: bool = true;
const ZEMP_ASSET_MIN_BALANCE: u128 = 1;

// ZEMP Asset Metadata
const ZEMP_ASSET_NAME: &str = "ZEMP";
const ZEMP_ASSET_SYMBOL: &str = "ZEMP";
const ZEMP_ASSET_NUMBER_OF_DECIMAL_PLACES: u8 = 18;

// Asset Units
const ONE_TEMP: u128 = 100_000_000;
const ONE_ZEMP: u128 = 1_000_000_000_000_000_000;
const ONE_TDFY: u128 = 1_000_000_000_000;

// Test Accounts
const ALICE_ACCOUNT_ID: AccountId = 1;
const BOB_ACCOUNT_ID: AccountId = 2;
const CHARLIE_ACCOUNT_ID: AccountId = 3;
const DAVE_ACCOUNT_ID: AccountId = 4;

// Extrinsic Hashes
const EXTRINSIC_HASH_0: [u8; 32] = [0; 32];
const EXTRINSIC_HASH_1: [u8; 32] = [1; 32];
const EXTRINSIC_HASH_2: [u8; 32] = [2; 32];

// Swap Fee Rates
const REQUESTER_SWAP_FEE_RATE: Permill = FeeAmount::get();
const MARKET_MAKER_SWAP_FEE_RATE: Permill = MarketMakerFeeAmount::get();

// Slippage Rates
const SLIPPAGE_0_PERCENT: Permill = Permill::from_percent(0);
const SLIPPAGE_2_PERCENTS: Permill = Permill::from_percent(2);
const SLIPPAGE_4_PERCENTS: Permill = Permill::from_percent(4);
const SLIPPAGE_5_PERCENTS: Permill = Permill::from_percent(5);

type BlockNumber = u64;

#[derive(Clone)]
struct Context {
  alice: Origin,
  bob: Origin,
  market_makers: Vec<AccountId>,
  fees_account_id: AccountId,
}

impl Default for Context {
  fn default() -> Self {
    let fees_account_id = Fees::account_id();
    assert_eq!(fees_account_id, 8246216774960574317);

    Self {
      alice: Origin::signed(ALICE_ACCOUNT_ID),
      bob: Origin::signed(BOB_ACCOUNT_ID),
      market_makers: vec![],
      fees_account_id: fees_account_id,
    }
  }
}

impl Context {
  fn set_oracle_status(self, status: bool) -> Self {
    assert_ok!(Oracle::set_status(self.alice.clone(), status));
    match status {
      true => assert!(Oracle::status()),
      false => assert!(!Oracle::status()),
    }
    self
  }

  fn set_market_makers(mut self, account_ids: Vec<AccountId>) -> Self {
    self.market_makers = account_ids;
    self
  }

  fn create_temp_asset_and_metadata(self) -> Self {
    let temp_asset_owner = ALICE_ACCOUNT_ID;

    assert_ok!(Assets::force_create(
      Origin::root(),
      TEMP_ASSET_ID,
      temp_asset_owner,
      TEMP_ASSET_IS_SUFFICIENT,
      TEMP_ASSET_MIN_BALANCE
    ));

    assert_ok!(Assets::set_metadata(
      Origin::signed(temp_asset_owner),
      TEMP_ASSET_ID,
      TEMP_ASSET_NAME.into(),
      TEMP_ASSET_SYMBOL.into(),
      TEMP_ASSET_NUMBER_OF_DECIMAL_PLACES
    ));

    self
  }

  fn create_temp2_asset_metadata(self) -> Self {
    let temp2_asset_owner = ALICE_ACCOUNT_ID;

    assert_ok!(Assets::force_create(
      Origin::root(),
      TEMP2_ASSET_ID,
      temp2_asset_owner,
      TEMP_ASSET_IS_SUFFICIENT,
      TEMP_ASSET_MIN_BALANCE
    ));

    assert_ok!(Assets::set_metadata(
      Origin::signed(temp2_asset_owner),
      TEMP2_ASSET_ID,
      TEMP2_ASSET_NAME.into(),
      TEMP2_ASSET_SYMBOL.into(),
      TEMP2_ASSET_NUMBER_OF_DECIMAL_PLACES
    ));

    self
  }

  fn create_zemp_asset_and_metadata(self) -> Self {
    let zemp_asset_owner = ALICE_ACCOUNT_ID;

    assert_ok!(Assets::force_create(
      Origin::root(),
      ZEMP_ASSET_ID,
      zemp_asset_owner,
      ZEMP_ASSET_IS_SUFFICIENT,
      ZEMP_ASSET_MIN_BALANCE
    ));

    assert_ok!(Assets::set_metadata(
      Origin::signed(zemp_asset_owner),
      ZEMP_ASSET_ID,
      ZEMP_ASSET_NAME.into(),
      ZEMP_ASSET_SYMBOL.into(),
      ZEMP_ASSET_NUMBER_OF_DECIMAL_PLACES
    ));

    self
  }

  fn mint_tdfy(self, account: AccountId, amount: u128) -> Self {
    Self::mint_asset_for_accounts(vec![account], CurrencyId::Tdfy, amount);
    assert_eq!(Adapter::balance(CurrencyId::Tdfy, &account), amount);
    self
  }

  fn mint_temp(self, account: AccountId, amount: u128) -> Self {
    Self::mint_asset_for_accounts(vec![account], TEMP_CURRENCY_ID, amount);
    assert_eq!(Adapter::balance(TEMP_CURRENCY_ID, &account), amount);
    self
  }

  fn mint_temp2(self, account: AccountId, amount: u128) -> Self {
    Self::mint_asset_for_accounts(vec![account], TEMP2_CURRENCY_ID, amount);
    assert_eq!(Adapter::balance(TEMP2_CURRENCY_ID, &account), amount);
    self
  }

  fn mint_zemp(self, account: AccountId, amount: u128) -> Self {
    Self::mint_asset_for_accounts(vec![account], ZEMP_CURRENCY_ID, amount);
    assert_eq!(Adapter::balance(ZEMP_CURRENCY_ID, &account), amount);
    self
  }

  fn mint_asset_for_accounts(accounts: Vec<AccountId>, asset: CurrencyId, amount: u128) {
    for account in accounts {
      assert_ok!(Adapter::mint_into(asset, &account, amount));
    }
  }

  fn create_tdfy_to_temp_limit_swap_request(
    &self,
    requester_account_id: AccountId,
    tdfy_amount: Balance,
    temp_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      CurrencyId::Tdfy,
      tdfy_amount,
      TEMP_CURRENCY_ID,
      temp_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      self.market_makers.contains(&requester_account_id),
      SwapType::Limit,
      slippage,
    )
  }

  fn create_temp_to_tdfy_limit_swap_request(
    &self,
    requester_account_id: AccountId,
    temp_amount: Balance,
    tdfy_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      TEMP_CURRENCY_ID,
      temp_amount,
      CurrencyId::Tdfy,
      tdfy_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      self.market_makers.contains(&requester_account_id),
      SwapType::Limit,
      slippage,
    )
  }

  fn create_temp_to_zemp_limit_swap_request(
    &self,
    requester_account_id: AccountId,
    temp_amount: Balance,
    zemp_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      TEMP_CURRENCY_ID,
      temp_amount,
      ZEMP_CURRENCY_ID,
      zemp_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      self.market_makers.contains(&requester_account_id),
      SwapType::Limit,
      slippage,
    )
  }

  fn create_tdfy_to_temp_market_swap_request(
    &self,
    requester_account_id: AccountId,
    tdfy_amount: Balance,
    temp_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      CurrencyId::Tdfy,
      tdfy_amount,
      TEMP_CURRENCY_ID,
      temp_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      self.market_makers.contains(&requester_account_id),
      SwapType::Market,
      slippage,
    )
  }

  fn create_zemp_to_temp_market_swap_request(
    &self,
    requester_account_id: AccountId,
    zemp_amount: Balance,
    temp_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      ZEMP_CURRENCY_ID,
      zemp_amount,
      TEMP_CURRENCY_ID,
      temp_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      self.market_makers.contains(&requester_account_id),
      SwapType::Market,
      slippage,
    )
  }

  fn create_zemp_to_temp_limit_swap_request(
    &self,
    requester_account_id: AccountId,
    zemp_amount: Balance,
    temp_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      ZEMP_CURRENCY_ID,
      zemp_amount,
      TEMP_CURRENCY_ID,
      temp_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      self.market_makers.contains(&requester_account_id),
      SwapType::Limit,
      slippage,
    )
  }

  fn create_temp_to_zemp_market_swap_request(
    &self,
    requester_account_id: AccountId,
    temp_amount: Balance,
    zemp_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      TEMP_CURRENCY_ID,
      temp_amount,
      ZEMP_CURRENCY_ID,
      zemp_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      self.market_makers.contains(&requester_account_id),
      SwapType::Market,
      slippage,
    )
  }

  fn create_temp_to_tdfy_market_swap_request(
    &self,
    requester_account_id: AccountId,
    temp_amount: Balance,
    tdfy_amount: Balance,
    extrinsic_hash: [u8; 32],
    slippage: Permill,
  ) -> Hash {
    add_new_swap_and_assert_results(
      requester_account_id,
      TEMP_CURRENCY_ID,
      temp_amount,
      CurrencyId::Tdfy,
      tdfy_amount,
      CURRENT_BLOCK_NUMBER,
      extrinsic_hash,
      self.market_makers.contains(&requester_account_id),
      SwapType::Market,
      slippage,
    )
  }
}

fn add_new_swap_and_assert_results(
  account_id: AccountId,
  asset_id_from: CurrencyId,
  amount_from: Balance,
  asset_id_to: CurrencyId,
  amount_to: Balance,
  block_number: BlockNumber,
  extrinsic_hash: [u8; 32],
  is_market_maker: bool,
  swap_type: SwapType,
  slippage: Permill,
) -> Hash {
  let initial_from_token_balance = Adapter::balance(asset_id_from, &account_id);

  let (trade_request_id, trade_request) = Oracle::add_new_swap_in_queue(
    account_id,
    asset_id_from,
    amount_from,
    asset_id_to,
    amount_to,
    block_number,
    extrinsic_hash,
    is_market_maker,
    swap_type,
    slippage,
  )
  .unwrap();

  assert_swap_cost_is_suspended(account_id, asset_id_from, amount_from, is_market_maker);

  if asset_id_from != CurrencyId::Tdfy {
    assert_sold_tokens_are_deducted(
      account_id,
      asset_id_from,
      initial_from_token_balance,
      amount_from,
      is_market_maker,
    );
  }

  assert_spendable_balance_is_updated(
    account_id,
    asset_id_from,
    initial_from_token_balance,
    amount_from,
    is_market_maker,
  );

  assert_eq!(trade_request.status, SwapStatus::Pending);
  assert_eq!(
    Oracle::account_swaps(account_id)
      .unwrap()
      .iter()
      .find(|(request_id, _)| *request_id == trade_request_id),
    Some(&(trade_request_id, SwapStatus::Pending))
  );

  assert_eq!(trade_request.block_number, CURRENT_BLOCK_NUMBER);

  trade_request_id
}

fn assert_swap_cost_is_suspended(
  account_id: AccountId,
  currency_id: CurrencyId,
  sell_amount: Balance,
  is_market_maker: bool,
) {
  let swap_fee_rate = if is_market_maker {
    MARKET_MAKER_SWAP_FEE_RATE
  } else {
    REQUESTER_SWAP_FEE_RATE
  };

  assert_eq!(
    Adapter::balance_on_hold(currency_id, &account_id),
    sell_amount.saturating_add(swap_fee_rate * sell_amount)
  );
}

fn assert_spendable_balance_is_updated(
  account_id: AccountId,
  currency_id: CurrencyId,
  initial_balance: Balance,
  sell_amount: Balance,
  is_market_maker: bool,
) {
  let swap_fee_rate = if is_market_maker {
    MARKET_MAKER_SWAP_FEE_RATE
  } else {
    REQUESTER_SWAP_FEE_RATE
  };

  let expected_reducible_balance = initial_balance
    .saturating_sub(sell_amount)
    .saturating_sub(swap_fee_rate * sell_amount);

  match currency_id {
    CurrencyId::Tdfy => assert_eq!(
      Adapter::reducible_balance(currency_id, &account_id, true),
      expected_reducible_balance
    ),
    _ => assert_eq!(
      Adapter::reducible_balance(currency_id, &account_id, true),
      expected_reducible_balance.saturating_sub(1_u128) // keep-alive token
    ),
  }

  assert_eq!(
    Adapter::reducible_balance(currency_id, &account_id, false),
    initial_balance
      .saturating_sub(sell_amount)
      .saturating_sub(swap_fee_rate * sell_amount)
  );
}

fn assert_sold_tokens_are_deducted(
  account_id: AccountId,
  currency_id: CurrencyId,
  initial_balance: Balance,
  sell_amount: Balance,
  is_market_maker: bool,
) {
  let swap_fee_rate = if is_market_maker {
    MARKET_MAKER_SWAP_FEE_RATE
  } else {
    REQUESTER_SWAP_FEE_RATE
  };

  assert_eq!(
    Adapter::balance(currency_id, &account_id),
    initial_balance
      .saturating_sub(sell_amount)
      .saturating_sub(swap_fee_rate * sell_amount)
  );
}

#[test]
pub fn check_genesis_config() {
  new_test_ext().execute_with(|| {
    assert!(!Oracle::status());
  });
}

#[test]
pub fn set_operational_status_works() {
  new_test_ext().execute_with(|| {
    let context = Context::default();

    assert_ok!(Oracle::set_status(context.alice.clone(), true));
    assert_noop!(
      Oracle::set_status(context.bob, false),
      Error::<Test>::AccessDenied
    );
    assert!(Oracle::status());
    assert_ok!(Oracle::set_status(context.alice, false));
    assert!(!Oracle::status());
  });
}

#[test]
pub fn confirm_swap_partial_filling() {
  new_test_ext().execute_with(|| {
    const BOB_INITIAL_20_TDFYS: Balance = 20 * ONE_TDFY;
    const CHARLIE_INITIAL_10000_TEMPS: Balance = 10_000 * ONE_TEMP;
    const DAVE_INITIAL_10000_TEMPS: Balance = 10_000 * ONE_TEMP;

    let context = Context::default()
      .set_oracle_status(true)
      .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
      .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
      .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
      .mint_tdfy(DAVE_ACCOUNT_ID, ONE_TDFY)
      .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
      .create_temp_asset_and_metadata()
      .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS)
      .mint_temp(DAVE_ACCOUNT_ID, DAVE_INITIAL_10000_TEMPS);

    const BOB_SELLS_10_TDFYS: Balance = 10 * ONE_TDFY;
    const BOB_BUYS_200_TEMPS: Balance = 200 * ONE_TEMP;
    let trade_request_id = context.create_tdfy_to_temp_limit_swap_request(
      BOB_ACCOUNT_ID,
      BOB_SELLS_10_TDFYS,
      BOB_BUYS_200_TEMPS,
      EXTRINSIC_HASH_0,
      SLIPPAGE_2_PERCENTS,
    );

    assert_eq!(
      trade_request_id,
      Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
        .unwrap_or_default()
    );

    const CHARLIE_SELLS_4000_TEMPS: Balance = 4_000 * ONE_TEMP;
    const CHARLIE_BUYS_200_TDFYS: Balance = 200 * ONE_TDFY;
    let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
      CHARLIE_ACCOUNT_ID,
      CHARLIE_SELLS_4000_TEMPS,
      CHARLIE_BUYS_200_TDFYS,
      EXTRINSIC_HASH_1,
      SLIPPAGE_4_PERCENTS,
    );

    assert_eq!(
      trade_request_mm_id,
      Hash::from_str("0x9ee76e89d3eae9ddad2e0b731e29ddcfa0781f7035600c5eb885637592e1d2c2")
        .unwrap_or_default()
    );

    const DAVE_SELLS_8000_TEMPS: Balance = 8_000 * ONE_TEMP;
    const DAVE_BUYS_400_TDFYS: Balance = 400 * ONE_TDFY;
    let trade_request_mm2_id = context.create_temp_to_tdfy_limit_swap_request(
      DAVE_ACCOUNT_ID,
      DAVE_SELLS_8000_TEMPS,
      DAVE_BUYS_400_TDFYS,
      EXTRINSIC_HASH_2,
      SLIPPAGE_5_PERCENTS,
    );

    const CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS: Balance = 100 * ONE_TEMP;
    const CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS: Balance = 5 * ONE_TDFY;
    // partial filling
    assert_ok!(Oracle::confirm_swap(
      context.alice.clone(),
      trade_request_id,
      vec![
        // charlie
        SwapConfirmation {
          request_id: trade_request_mm_id,
          amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
          amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
        },
      ],
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
      BOB_INITIAL_20_TDFYS
        .saturating_sub(CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS)
        .saturating_sub(REQUESTER_SWAP_FEE_RATE * CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS)
    );

    // swap confirmation for bob (user)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_id,
      status: SwapStatus::PartiallyFilled,
      account_id: BOB_ACCOUNT_ID,
      currency_from: CurrencyId::Tdfy,
      currency_amount_from: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
      currency_to: TEMP_CURRENCY_ID,
      currency_amount_to: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
      initial_extrinsic_hash: EXTRINSIC_HASH_0,
    }));

    // swap confirmation for charlie (mm)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_mm_id,
      status: SwapStatus::PartiallyFilled,
      account_id: CHARLIE_ACCOUNT_ID,
      currency_from: TEMP_CURRENCY_ID,
      currency_amount_from: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
      currency_to: CurrencyId::Tdfy,
      currency_amount_to: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
      initial_extrinsic_hash: EXTRINSIC_HASH_1,
    }));

    // BOB: make sure the CLIENT current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::swaps(trade_request_id).unwrap();
    assert_eq!(trade_request_filled.status, SwapStatus::PartiallyFilled);

    let trade_request_account = Oracle::account_swaps(BOB_ACCOUNT_ID).unwrap();
    assert_eq!(
      trade_request_account
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_id),
      Some(&(trade_request_id, SwapStatus::PartiallyFilled))
    );

    assert_eq!(
      trade_request_filled.amount_from_filled,
      CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS
    );
    assert_eq!(
      trade_request_filled.amount_to_filled,
      CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
    );

    // CHARLIE: make sure the MM current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::swaps(trade_request_mm_id).unwrap();
    assert_eq!(trade_request_filled.status, SwapStatus::PartiallyFilled);
    assert_eq!(
      trade_request_filled.amount_from_filled,
      CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
    );
    assert_eq!(
      trade_request_filled.amount_to_filled,
      CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS
    );

    const DAVE_PARTIAL_FILLING_SELLS_100_TEMPS: Balance = 100 * ONE_TEMP;
    const DAVE_PARTIAL_FILLING_BUYS_5_TDFYS: Balance = 5 * ONE_TDFY;

    // another partial filling who should close the trade
    assert_ok!(Oracle::confirm_swap(
      context.alice.clone(),
      trade_request_id,
      vec![SwapConfirmation {
        request_id: trade_request_mm2_id,
        amount_to_receive: DAVE_PARTIAL_FILLING_BUYS_5_TDFYS,
        amount_to_send: DAVE_PARTIAL_FILLING_SELLS_100_TEMPS,
      },],
    ));

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
      BOB_INITIAL_20_TDFYS
        .saturating_sub(10 * ONE_TDFY)
        .saturating_sub(REQUESTER_SWAP_FEE_RATE * (10 * ONE_TDFY))
    );

    // swap confirmation for bob (user)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_id,
      status: SwapStatus::Completed,
      account_id: BOB_ACCOUNT_ID,
      currency_from: CurrencyId::Tdfy,
      currency_amount_from: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
      currency_to: TEMP_CURRENCY_ID,
      currency_amount_to: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
      initial_extrinsic_hash: EXTRINSIC_HASH_0,
    }));

    // swap confirmation for dave (second mm)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_mm2_id,
      status: SwapStatus::PartiallyFilled,
      account_id: DAVE_ACCOUNT_ID,
      currency_from: TEMP_CURRENCY_ID,
      currency_amount_from: DAVE_PARTIAL_FILLING_SELLS_100_TEMPS,
      currency_to: CurrencyId::Tdfy,
      currency_amount_to: DAVE_PARTIAL_FILLING_BUYS_5_TDFYS,
      initial_extrinsic_hash: EXTRINSIC_HASH_2,
    }));

    // BOB: make sure the CLIENT current trade is deleted
    assert!(Oracle::swaps(trade_request_id).is_none());
    let trade_request_account = Oracle::account_swaps(BOB_ACCOUNT_ID).unwrap();
    assert_eq!(
      trade_request_account
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_id),
      None
    );

    // cant send another trade confirmation as the request should be deleted
    // we do expect `InvalidRequestId`
    assert_noop!(
      Oracle::confirm_swap(context.alice.clone(), trade_request_id, vec![],),
      Error::<Test>::InvalidRequestId
    );

    // DAVE: make sure the MM current trade is partially filled and correctly updated
    let trade_request_filled = Oracle::swaps(trade_request_mm2_id).unwrap();
    assert_eq!(trade_request_filled.status, SwapStatus::PartiallyFilled);
    assert_eq!(
      trade_request_filled.amount_from_filled,
      DAVE_PARTIAL_FILLING_SELLS_100_TEMPS
    );
    assert_eq!(
      trade_request_filled.amount_to_filled,
      DAVE_PARTIAL_FILLING_BUYS_5_TDFYS
    );

    // cancel our mm's swap to release the funds
    assert_ok!(Oracle::cancel_swap(
      context.alice.clone(),
      trade_request_mm_id,
    ));
    assert_ok!(Oracle::cancel_swap(context.alice, trade_request_mm2_id,));

    // validate all balance
    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
      BOB_INITIAL_20_TDFYS
        .saturating_sub(BOB_SELLS_10_TDFYS)
        .saturating_sub(REQUESTER_SWAP_FEE_RATE * BOB_SELLS_10_TDFYS)
    );
    assert_eq!(
      Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
      BOB_BUYS_200_TEMPS
    );
    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
      Zero::zero()
    );

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &CHARLIE_ACCOUNT_ID),
      // initial balance + swap
      ONE_TDFY + CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS
    );

    assert_spendable_balance_is_updated(
      CHARLIE_ACCOUNT_ID,
      TEMP_CURRENCY_ID,
      CHARLIE_INITIAL_10000_TEMPS,
      CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
      true,
    );

    assert_eq!(
      Adapter::balance_on_hold(TEMP_CURRENCY_ID, &CHARLIE_ACCOUNT_ID),
      Zero::zero()
    );

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &DAVE_ACCOUNT_ID),
      // initial balance + swap
      ONE_TDFY + DAVE_PARTIAL_FILLING_BUYS_5_TDFYS
    );

    assert_eq!(
      Adapter::balance(TEMP_CURRENCY_ID, &DAVE_ACCOUNT_ID),
      DAVE_INITIAL_10000_TEMPS
        .saturating_sub(DAVE_PARTIAL_FILLING_SELLS_100_TEMPS)
        .saturating_sub(MARKET_MAKER_SWAP_FEE_RATE * DAVE_PARTIAL_FILLING_SELLS_100_TEMPS)
    );

    assert_eq!(
      Adapter::balance_on_hold(TEMP_CURRENCY_ID, &DAVE_ACCOUNT_ID),
      Zero::zero()
    );
  });
}

#[test]
pub fn confirm_swap_temp_zemp() {
  new_test_ext().execute_with(|| {
    const BOB_INITIAL_ZEMPS: Balance = 10 * ONE_ZEMP;
    const CHARLIE_INITIAL_TEMPS: Balance = 2 * ONE_TEMP;

    let context = Context::default()
      .set_oracle_status(true)
      .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
      .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
      .mint_tdfy(BOB_ACCOUNT_ID, ONE_TDFY)
      .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
      .create_temp_asset_and_metadata()
      .create_zemp_asset_and_metadata()
      .mint_zemp(BOB_ACCOUNT_ID, BOB_INITIAL_ZEMPS)
      .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_TEMPS);

    const BOB_SELLS_ZEMPS: Balance = 4_889_975_550_122_249_389;
    const BOB_BUYS_TEMPS: Balance = 40_000_000;
    let trade_request_id = context.create_zemp_to_temp_market_swap_request(
      BOB_ACCOUNT_ID,
      BOB_SELLS_ZEMPS,
      BOB_BUYS_TEMPS,
      EXTRINSIC_HASH_0,
      SLIPPAGE_2_PERCENTS,
    );

    assert_eq!(
      trade_request_id,
      Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
        .unwrap_or_default()
    );

    // 0.80478930
    const CHARLIE_SELLS_TEMPS: Balance = 80_478_930;
    // 9.838500000000000000
    const CHARLIE_BUYS_ZEMPS: Balance = 9_838_500_000_000_000_000;

    let trade_request_mm_id = context.create_temp_to_zemp_limit_swap_request(
      CHARLIE_ACCOUNT_ID,
      CHARLIE_SELLS_TEMPS,
      CHARLIE_BUYS_ZEMPS,
      EXTRINSIC_HASH_1,
      SLIPPAGE_0_PERCENT,
    );

    assert_eq!(
      trade_request_mm_id,
      Hash::from_str("0x9ee76e89d3eae9ddad2e0b731e29ddcfa0781f7035600c5eb885637592e1d2c2")
        .unwrap_or_default()
    );

    // partial filling
    assert_ok!(Oracle::confirm_swap(
      context.alice.clone(),
      trade_request_id,
      vec![
        // charlie
        SwapConfirmation {
          request_id: trade_request_mm_id,
          amount_to_receive: BOB_SELLS_ZEMPS,
          amount_to_send: BOB_BUYS_TEMPS,
        },
      ],
    ));
  });
}

#[test]
pub fn confirm_swap_zemp_temp() {
  new_test_ext().execute_with(|| {
    const BOB_INITIAL_TEMPS: Balance = 900_000 * ONE_TEMP;
    const CHARLIE_INITIAL_ZEMPS: Balance = 900_000 * ONE_ZEMP;

    let context = Context::default()
      .set_oracle_status(true)
      .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
      .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
      .mint_tdfy(BOB_ACCOUNT_ID, ONE_TDFY)
      .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
      .create_temp_asset_and_metadata()
      .create_zemp_asset_and_metadata()
      .mint_temp(BOB_ACCOUNT_ID, BOB_INITIAL_TEMPS)
      .mint_zemp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_ZEMPS);

    const BOB_SELLS_TEMPS: Balance = 10_000_000;
    const BOB_BUYS_ZEMPS: Balance = 12_500_000_000_000_000_000;

    let trade_request_id = context.create_temp_to_zemp_market_swap_request(
      BOB_ACCOUNT_ID,
      BOB_SELLS_TEMPS,
      BOB_BUYS_ZEMPS,
      EXTRINSIC_HASH_0,
      SLIPPAGE_2_PERCENTS,
    );

    assert_eq!(
      trade_request_id,
      Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
        .unwrap_or_default()
    );

    // 0.80478930
    const CHARLIE_SELLS_ZEMPS: Balance = 1_000_000_000_000_000_000_000;
    // 9.838500000000000000
    const CHARLIE_BUYS_TEMPS: Balance = 800_000_000;

    let trade_request_mm_id = context.create_zemp_to_temp_limit_swap_request(
      CHARLIE_ACCOUNT_ID,
      CHARLIE_SELLS_ZEMPS,
      CHARLIE_BUYS_TEMPS,
      EXTRINSIC_HASH_1,
      SLIPPAGE_0_PERCENT,
    );

    assert_eq!(
      trade_request_mm_id,
      Hash::from_str("0x9ee76e89d3eae9ddad2e0b731e29ddcfa0781f7035600c5eb885637592e1d2c2")
        .unwrap_or_default()
    );

    // partial filling
    assert_ok!(Oracle::confirm_swap(
      context.alice.clone(),
      trade_request_id,
      vec![
        // charlie
        SwapConfirmation {
          request_id: trade_request_mm_id,
          amount_to_receive: BOB_SELLS_TEMPS,
          amount_to_send: BOB_BUYS_ZEMPS,
        },
      ],
    ));
  });
}

#[test]
pub fn confirm_swap_with_fees() {
  new_test_ext().execute_with(|| {
    const BOB_INITIAL_20_TDFYS: Balance = 20 * ONE_TDFY;
    const CHARLIE_INITIAL_10000_TEMPS: Balance = 10_000 * ONE_TEMP;
    const DAVE_INITIAL_10000_TEMPS: Balance = 10_000 * ONE_TEMP;

    let context = Context::default()
      .set_oracle_status(true)
      .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
      .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
      .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
      .mint_tdfy(DAVE_ACCOUNT_ID, ONE_TDFY)
      .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
      .create_temp_asset_and_metadata()
      .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS)
      .mint_temp(DAVE_ACCOUNT_ID, DAVE_INITIAL_10000_TEMPS);

    Fees::start_era();
    assert!(!Fees::current_era().is_none());
    let current_era = Fees::current_era().unwrap().index;

    const BOB_SELLS_10_TDFYS: Balance = 10 * ONE_TDFY;
    const BOB_BUYS_200_TEMPS: Balance = 200 * ONE_TEMP;
    let trade_request_id = context.create_tdfy_to_temp_limit_swap_request(
      BOB_ACCOUNT_ID,
      BOB_SELLS_10_TDFYS,
      BOB_BUYS_200_TEMPS,
      EXTRINSIC_HASH_0,
      SLIPPAGE_2_PERCENTS,
    );

    assert_eq!(
      trade_request_id,
      Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
        .unwrap_or_default()
    );

    const CHARLIE_SELLS_4000_TEMPS: Balance = 4_000 * ONE_TEMP;
    const CHARLIE_BUYS_200_TDFYS: Balance = 200 * ONE_TDFY;
    let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
      CHARLIE_ACCOUNT_ID,
      CHARLIE_SELLS_4000_TEMPS,
      CHARLIE_BUYS_200_TDFYS,
      EXTRINSIC_HASH_1,
      SLIPPAGE_5_PERCENTS,
    );

    assert_eq!(
      trade_request_mm_id,
      Hash::from_str("0x9ee76e89d3eae9ddad2e0b731e29ddcfa0781f7035600c5eb885637592e1d2c2")
        .unwrap_or_default()
    );

    const DAVE_SELLS_100_TEMPS: Balance = 100 * ONE_TEMP;
    const DAVE_BUYS_5_TDFYS: Balance = 5 * ONE_TDFY;
    let trade_request_mm2_id = context.create_temp_to_tdfy_limit_swap_request(
      DAVE_ACCOUNT_ID,
      DAVE_SELLS_100_TEMPS,
      DAVE_BUYS_5_TDFYS,
      EXTRINSIC_HASH_2,
      SLIPPAGE_4_PERCENTS,
    );

    // partial fillings
    const CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS: Balance = 5 * ONE_TDFY;
    const CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS: Balance = 100 * ONE_TEMP;
    const DAVE_PARTIAL_FILLING_BUYS_5_TDFYS: Balance = 5 * ONE_TDFY;
    const DAVE_PARTIAL_FILLING_SELLS_100_TEMPS: Balance = 100 * ONE_TEMP;

    assert_ok!(Oracle::confirm_swap(
      context.alice,
      trade_request_id,
      vec![
        SwapConfirmation {
          request_id: trade_request_mm_id,
          amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
          amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
        },
        SwapConfirmation {
          request_id: trade_request_mm2_id,
          amount_to_receive: DAVE_PARTIAL_FILLING_BUYS_5_TDFYS,
          amount_to_send: DAVE_PARTIAL_FILLING_SELLS_100_TEMPS,
        },
      ],
    ));

    // swap confirmation for bob (user)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_id,
      status: SwapStatus::Completed,
      account_id: BOB_ACCOUNT_ID,
      currency_from: CurrencyId::Tdfy,
      currency_amount_from: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS
        + DAVE_PARTIAL_FILLING_BUYS_5_TDFYS,
      currency_to: TEMP_CURRENCY_ID,
      currency_amount_to: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
        + DAVE_PARTIAL_FILLING_SELLS_100_TEMPS,
      initial_extrinsic_hash: EXTRINSIC_HASH_0,
    }));

    // swap confirmation for charlie (mm1)
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_mm_id,
      status: SwapStatus::PartiallyFilled,
      account_id: CHARLIE_ACCOUNT_ID,
      currency_from: TEMP_CURRENCY_ID,
      currency_amount_from: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
      currency_to: CurrencyId::Tdfy,
      currency_amount_to: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
      initial_extrinsic_hash: EXTRINSIC_HASH_1,
    }));

    // swap confirmation for dave (mm2)
    // the trade should be closed, because amount_from of the request is filled
    System::assert_has_event(MockEvent::Oracle(Event::SwapProcessed {
      request_id: trade_request_mm2_id,
      status: SwapStatus::Completed,
      account_id: DAVE_ACCOUNT_ID,
      currency_from: TEMP_CURRENCY_ID,
      currency_amount_from: DAVE_PARTIAL_FILLING_SELLS_100_TEMPS,
      currency_to: CurrencyId::Tdfy,
      currency_amount_to: DAVE_PARTIAL_FILLING_BUYS_5_TDFYS,
      initial_extrinsic_hash: EXTRINSIC_HASH_2,
    }));

    // BOB: make sure the CLIENT current trade is deleted
    assert!(Oracle::swaps(trade_request_id).is_none());
    assert_eq!(
      Oracle::account_swaps(BOB_ACCOUNT_ID)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_id),
      None
    );

    // CHARLIE: make sure the MM current trade is partially filled and correctly updated
    assert_eq!(
      Oracle::account_swaps(CHARLIE_ACCOUNT_ID)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_mm_id),
      Some(&(trade_request_mm_id, SwapStatus::PartiallyFilled))
    );

    let trade_request_filled = Oracle::swaps(trade_request_mm_id).unwrap();
    assert_eq!(trade_request_filled.status, SwapStatus::PartiallyFilled);
    assert_eq!(
      trade_request_filled.amount_from_filled,
      CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
    );
    assert_eq!(
      trade_request_filled.amount_to_filled,
      CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS
    );

    // DAVE: make sure the MM current trade is totally filled (deleted)
    assert!(Oracle::swaps(trade_request_mm2_id).is_none());
    assert_eq!(
      Oracle::account_swaps(DAVE_ACCOUNT_ID)
        .unwrap()
        .iter()
        .find(|(request_id, _)| *request_id == trade_request_mm2_id),
      None
    );

    // make sure all balances match
    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &context.fees_account_id),
      // we burned 1 tdfy on start so it should contain 1.2 tdfy now
      ONE_TDFY + REQUESTER_SWAP_FEE_RATE * BOB_SELLS_10_TDFYS
    );

    assert_eq!(
      Adapter::balance(TEMP_CURRENCY_ID, &context.fees_account_id),
      MARKET_MAKER_SWAP_FEE_RATE
        * (CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS + DAVE_PARTIAL_FILLING_SELLS_100_TEMPS)
    );

    assert_eq!(
      Adapter::balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
      BOB_INITIAL_20_TDFYS
        .saturating_sub(BOB_SELLS_10_TDFYS)
        .saturating_sub(REQUESTER_SWAP_FEE_RATE * BOB_SELLS_10_TDFYS)
    );

    assert_eq!(
      Adapter::balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
      BOB_BUYS_200_TEMPS
    );

    // make sure fees are registered on chain
    let bob_fee = Fees::account_fees(current_era, BOB_ACCOUNT_ID);
    assert_eq!(
      bob_fee.first().unwrap().1.fee,
      REQUESTER_SWAP_FEE_RATE * BOB_SELLS_10_TDFYS
    );
    assert_eq!(bob_fee.first().unwrap().1.amount, BOB_SELLS_10_TDFYS);

    let charlie_fee = Fees::account_fees(current_era, CHARLIE_ACCOUNT_ID);
    assert_eq!(
      charlie_fee.first().unwrap().1.fee,
      MARKET_MAKER_SWAP_FEE_RATE * CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
    );
    assert_eq!(
      charlie_fee.first().unwrap().1.amount,
      CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS
    );

    let dave_fee = Fees::account_fees(current_era, DAVE_ACCOUNT_ID);
    assert_eq!(
      dave_fee.first().unwrap().1.fee,
      MARKET_MAKER_SWAP_FEE_RATE * DAVE_PARTIAL_FILLING_SELLS_100_TEMPS
    );
    assert_eq!(
      dave_fee.first().unwrap().1.amount,
      DAVE_PARTIAL_FILLING_SELLS_100_TEMPS
    );
  });
}

#[test]
pub fn confirm_swap_ourself() {
  new_test_ext().execute_with(|| {
    const BOB_INITIAL_20_TDFYS: Balance = 20 * ONE_TDFY;
    const BOB_INITIAL_10000_TEMPS: Balance = 10_000 * ONE_TEMP;

    let context = Context::default()
      .set_oracle_status(true)
      .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
      .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
      .create_temp_asset_and_metadata()
      .mint_temp(BOB_ACCOUNT_ID, BOB_INITIAL_10000_TEMPS);

    const BOB_SELLS_10_TDFYS: Balance = 10 * ONE_TDFY;
    const BOB_BUYS_400_TEMPS: Balance = 400 * ONE_TEMP;
    let trade_request_id = context.create_tdfy_to_temp_limit_swap_request(
      BOB_ACCOUNT_ID,
      BOB_SELLS_10_TDFYS,
      BOB_BUYS_400_TEMPS,
      EXTRINSIC_HASH_0,
      SLIPPAGE_2_PERCENTS,
    );

    assert_eq!(
      trade_request_id,
      Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
        .unwrap_or_default()
    );

    const BOB_SELLS_400_TEMPS: Balance = 400 * ONE_TEMP;
    const BOB_BUYS_10_TDFYS: Balance = 10 * ONE_TDFY;
    let context = Context::default().set_market_makers(vec![BOB_ACCOUNT_ID]);
    let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
      BOB_ACCOUNT_ID,
      BOB_SELLS_400_TEMPS,
      BOB_BUYS_10_TDFYS,
      EXTRINSIC_HASH_0,
      SLIPPAGE_5_PERCENTS,
    );

    assert_eq!(
      trade_request_mm_id,
      Hash::from_str("0xe0424aac19ef997f1b76ac20d400aecc2ee0258d9eacb7013c3fcfa2e55bdc67")
        .unwrap_or_default()
    );

    // partial filling
    const BOB_FILLING_BUYS_10_TDFYS: Balance = 10 * ONE_TDFY;
    const BOB_FILLING_SELLS_400_TDFYS: Balance = 400 * ONE_TEMP;
    assert_ok!(Oracle::confirm_swap(
      context.alice.clone(),
      trade_request_id,
      vec![SwapConfirmation {
        request_id: trade_request_mm_id,
        amount_to_receive: BOB_FILLING_BUYS_10_TDFYS,
        amount_to_send: BOB_FILLING_SELLS_400_TDFYS,
      },],
    ));

    // BOB: make sure the CLIENT current trade is partially filled and correctly updated
    assert!(Oracle::swaps(trade_request_id).is_none());
    assert!(Oracle::swaps(trade_request_mm_id).is_none());

    // cant send another trade confirmation as the request should be deleted
    // we do expect `InvalidRequestId`
    assert_noop!(
      Oracle::confirm_swap(context.alice, trade_request_id, vec![],),
      Error::<Test>::InvalidRequestId
    );

    // validate all balance
    assert_eq!(
      Adapter::reducible_balance(CurrencyId::Tdfy, &BOB_ACCOUNT_ID, false),
      // we should refund the extra fees paid on the slippage value
      BOB_INITIAL_20_TDFYS.saturating_sub(REQUESTER_SWAP_FEE_RATE * BOB_SELLS_10_TDFYS)
    );

    assert_eq!(
      Adapter::reducible_balance(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID, false),
      BOB_INITIAL_10000_TEMPS.saturating_sub(MARKET_MAKER_SWAP_FEE_RATE * BOB_SELLS_400_TEMPS)
    );

    assert_eq!(
      Adapter::balance_on_hold(CurrencyId::Tdfy, &BOB_ACCOUNT_ID),
      Zero::zero()
    );
    assert_eq!(
      Adapter::balance_on_hold(TEMP_CURRENCY_ID, &BOB_ACCOUNT_ID),
      Zero::zero()
    );
  });
}

#[test]
pub fn test_slippage() {
  new_test_ext().execute_with(|| {
    const BOB_INITIAL_20_TDFYS: Balance = 20 * ONE_TDFY;
    const BOB_INITIAL_10000_TEMPS: Balance = 10_000 * ONE_TEMP;

    let context = Context::default()
      .set_oracle_status(true)
      .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
      .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
      .create_temp_asset_and_metadata()
      .mint_temp(BOB_ACCOUNT_ID, BOB_INITIAL_10000_TEMPS);

    const BOB_SELLS_10_TDFYS: Balance = 10 * ONE_TDFY;
    const BOB_BUYS_400_TEMPS: Balance = 400 * ONE_TEMP;
    let trade_request_id = context.create_tdfy_to_temp_market_swap_request(
      BOB_ACCOUNT_ID,
      BOB_SELLS_10_TDFYS,
      BOB_BUYS_400_TEMPS,
      EXTRINSIC_HASH_0,
      SLIPPAGE_2_PERCENTS,
    );

    assert_eq!(
      trade_request_id,
      Hash::from_str("0xd22a9d9ea0e217ddb07923d83c86f89687b682d1f81bb752d60b54abda0e7a3e")
        .unwrap_or_default()
    );

    let context = Context::default().set_market_makers(vec![BOB_ACCOUNT_ID]);
    const MM_BOB_SELLS_500_TEMPS: Balance = 500 * ONE_TEMP;
    const MM_BOB_BUYS_10_TDFYS: Balance = 10 * ONE_TDFY;
    let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
      BOB_ACCOUNT_ID,
      // ratio is a bit different (mm is willing to pay a bit more for the same amount)
      MM_BOB_SELLS_500_TEMPS,
      MM_BOB_BUYS_10_TDFYS,
      EXTRINSIC_HASH_0,
      SLIPPAGE_0_PERCENT,
    );

    assert_eq!(
      trade_request_mm_id,
      Hash::from_str("0xe0424aac19ef997f1b76ac20d400aecc2ee0258d9eacb7013c3fcfa2e55bdc67")
        .unwrap_or_default()
    );

    assert_noop!(
      Oracle::confirm_swap(
        context.alice.clone(),
        trade_request_id,
        vec![SwapConfirmation {
          request_id: trade_request_mm_id,
          amount_to_receive: BOB_SELLS_10_TDFYS.saturating_add(ONE_TDFY),
          amount_to_send: BOB_BUYS_400_TEMPS,
        },],
      ),
      Error::<Test>::OfferIsLessThanSwapLowerBound { index: 0 }
    );

    // partial filling
    assert_ok!(Oracle::confirm_swap(
      context.alice.clone(),
      trade_request_id,
      vec![SwapConfirmation {
        request_id: trade_request_mm_id,
        amount_to_receive: BOB_SELLS_10_TDFYS,
        amount_to_send: BOB_BUYS_400_TEMPS.saturating_sub(SLIPPAGE_2_PERCENTS * BOB_BUYS_400_TEMPS),
      },],
    ));

    // market order got deleted
    assert!(Oracle::swaps(trade_request_id).is_none());
    // limit order isnt deleted as its not fully filled
    assert!(Oracle::swaps(trade_request_mm_id).is_some());
  });
}

#[test]
pub fn test_imalive() {
  new_test_ext().execute_with(|| {
    let context = Context::default()
      .set_oracle_status(true)
      .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
      .create_temp_asset_and_metadata()
      .create_zemp_asset_and_metadata();

    assert_ok!(Oracle::update_assets_value(
      context.alice.clone(),
      vec![
        // 10 Tdfy / USDT
        (4, 10_000_000_000_000_u128),
        // 100k Tdfy / BTC
        (2, 100_000_000_000_000_000_u128),
      ]
    ));

    let fee =
      Fees::calculate_swap_fees(CurrencyId::Wrapped(4), 100_000_000, SwapType::Limit, false);
    assert_eq!(
      Sunrise::calculate_rebates_on_fees_paid(
        // 125%
        FixedU128::saturating_from_rational(125, 100),
        // 2$ USDT in fee
        // Should have total 2.5$ USDT in reward
        // 2.5 / 0.1 = 25 TDFY final
        &fee,
      )
      .unwrap(),
      25_000_000_000_000
    );
  });
}

mod confirm_swap {
  use super::*;

  const BOB_INITIAL_20_TDFYS: Balance = 20 * ONE_TDFY;
  const BOB_SELLS_10_TDFYS: Balance = 10 * ONE_TDFY;
  const BOB_BUYS_200_TEMPS: Balance = 200 * ONE_TEMP;

  const CHARLIE_INITIAL_10000_TEMPS: Balance = 10_000 * ONE_TEMP;
  const CHARLIE_SELLS_4000_TEMPS: Balance = 4_000 * ONE_TEMP;
  const CHARLIE_BUYS_200_TDFYS: Balance = 200 * ONE_TDFY;

  const CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS: Balance = 100 * ONE_TEMP;
  const CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS: Balance = 5 * ONE_TDFY;

  fn create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
    context: &Context,
  ) -> Hash {
    context.create_tdfy_to_temp_limit_swap_request(
      BOB_ACCOUNT_ID,
      BOB_SELLS_10_TDFYS,
      BOB_BUYS_200_TEMPS,
      EXTRINSIC_HASH_0,
      SLIPPAGE_2_PERCENTS,
    )
  }

  fn create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_5_percents_slippage(
    context: &Context,
  ) -> Hash {
    context.create_tdfy_to_temp_limit_swap_request(
      BOB_ACCOUNT_ID,
      BOB_SELLS_10_TDFYS,
      BOB_BUYS_200_TEMPS,
      EXTRINSIC_HASH_0,
      SLIPPAGE_5_PERCENTS,
    )
  }

  fn create_bob_market_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
    context: &Context,
  ) -> Hash {
    context.create_tdfy_to_temp_market_swap_request(
      BOB_ACCOUNT_ID,
      BOB_SELLS_10_TDFYS,
      BOB_BUYS_200_TEMPS,
      EXTRINSIC_HASH_0,
      SLIPPAGE_2_PERCENTS,
    )
  }

  fn create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_2_percents_slippage(
    context: &Context,
  ) -> Hash {
    context.create_temp_to_tdfy_limit_swap_request(
      CHARLIE_ACCOUNT_ID,
      CHARLIE_SELLS_4000_TEMPS,
      CHARLIE_BUYS_200_TDFYS,
      EXTRINSIC_HASH_1,
      SLIPPAGE_2_PERCENTS,
    )
  }

  fn create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
    context: &Context,
  ) -> Hash {
    context.create_temp_to_tdfy_limit_swap_request(
      CHARLIE_ACCOUNT_ID,
      CHARLIE_SELLS_4000_TEMPS,
      CHARLIE_BUYS_200_TDFYS,
      EXTRINSIC_HASH_1,
      SLIPPAGE_4_PERCENTS,
    )
  }

  fn create_charlie_market_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
    context: &Context,
  ) -> Hash {
    context.create_temp_to_tdfy_market_swap_request(
      CHARLIE_ACCOUNT_ID,
      CHARLIE_SELLS_4000_TEMPS,
      CHARLIE_BUYS_200_TDFYS,
      EXTRINSIC_HASH_1,
      SLIPPAGE_4_PERCENTS,
    )
  }

  mod fails_when {
    use super::*;

    #[test]
    fn oracle_is_paused() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(false)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::OraclePaused
        );
      });
    }

    #[test]
    fn not_signed() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            Origin::none(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          BadOrigin
        );
      });
    }

    #[test]
    fn not_signed_by_sender() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            context.bob.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::AccessDenied
        );
      });
    }

    #[test]
    fn request_id_is_invalid() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        const INVALID_REQUEST_ID: H256 = H256::zero();
        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            INVALID_REQUEST_ID,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::InvalidRequestId
        );
      });
    }

    #[test]
    fn trade_request_status_is_invalid() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        for invalid_status in vec![
          SwapStatus::Cancelled,
          SwapStatus::Completed,
          SwapStatus::Rejected,
        ] {
          Swaps::<Test>::mutate(trade_request_id, |request| {
            if let Some(trade_request) = request {
              trade_request.status = invalid_status.clone()
            }
          });

          assert_noop!(
            Oracle::confirm_swap(
              context.alice.clone(),
              trade_request_id,
              vec![SwapConfirmation {
                request_id: trade_request_mm_id,
                amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
                amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
              },],
            ),
            Error::<Test>::InvalidSwapRequestStatus
          );
        }
      });
    }

    #[test]
    fn market_maker_request_status_is_invalid() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        for invalid_status in vec![
          SwapStatus::Cancelled,
          SwapStatus::Completed,
          SwapStatus::Rejected,
        ] {
          Swaps::<Test>::mutate(trade_request_mm_id, |request| {
            if let Some(trade_request) = request {
              trade_request.status = invalid_status.clone()
            }
          });

          assert_noop!(
            Oracle::confirm_swap(
              context.alice.clone(),
              trade_request_id,
              vec![SwapConfirmation {
                request_id: trade_request_mm_id,
                amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
                amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
              },],
            ),
            Error::<Test>::InvalidMarketMakerSwapRequestStatus
          );
        }
      });
    }

    #[test]
    fn market_maker_request_id_is_invalid() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );

        const INVALID_REQUEST_ID: H256 = H256::zero();

        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: INVALID_REQUEST_ID,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::InvalidMarketMakerRequestId { index: 0 }
        );
      });
    }

    #[test]
    fn offer_is_less_than_swap_lower_bound() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_market_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );

        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_2_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: BOB_SELLS_10_TDFYS
                .saturating_add(SLIPPAGE_2_PERCENTS * BOB_SELLS_10_TDFYS)
                .saturating_add(ONE_TDFY),
              amount_to_send: BOB_BUYS_200_TEMPS,
            }],
          ),
          Error::<Test>::OfferIsLessThanSwapLowerBound { index: 0 }
        );
      });
    }

    #[test]
    fn offer_is_greater_than_swap_upper_bound() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: ONE_TDFY,
              amount_to_send: BOB_BUYS_200_TEMPS,
            }],
          ),
          Error::<Test>::OfferIsGreaterThanSwapUpperBound { index: 0 }
        );
      });
    }

    #[test]
    fn offer_is_less_than_market_maker_swap_lower_bound() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_5_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_market_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: BOB_SELLS_10_TDFYS,
              amount_to_send: BOB_BUYS_200_TEMPS
                .saturating_sub(SLIPPAGE_4_PERCENTS * BOB_BUYS_200_TEMPS)
                .saturating_sub(ONE_TEMP),
            }],
          ),
          Error::<Test>::OfferIsLessThanMarketMakerSwapLowerBound { index: 0 }
        );
      });
    }

    #[test]
    fn offer_is_greater_than_market_maker_swap_upper_bound() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID, DAVE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_5_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: BOB_SELLS_10_TDFYS,
              amount_to_send: BOB_BUYS_200_TEMPS
                .saturating_add(SLIPPAGE_5_PERCENTS * BOB_BUYS_200_TEMPS),
            },],
          ),
          Error::<Test>::OfferIsGreaterThanMarketMakerSwapUpperBound { index: 0 }
        );
      });
    }

    #[test]
    fn market_maker_does_not_have_enough_funds() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
          CHARLIE_ACCOUNT_ID,
          BOB_BUYS_200_TEMPS.saturating_div(5),
          BOB_SELLS_10_TDFYS.saturating_div(5),
          EXTRINSIC_HASH_1,
          SLIPPAGE_4_PERCENTS,
        );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
            },],
          ),
          Error::<Test>::MarketMakerHasNotEnoughTokenToSell
        );
      });
    }

    #[test]
    fn market_maker_overflow() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );

        let trade_request_mm_id = context.create_temp_to_tdfy_limit_swap_request(
          CHARLIE_ACCOUNT_ID,
          CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS,
          CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
          EXTRINSIC_HASH_1,
          SLIPPAGE_2_PERCENTS,
        );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS,
              // this pass the slippage but overflow the initial request
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS.saturating_add(1),
            },],
          ),
          Error::<Test>::MarketMakerHasNotEnoughTokenToSell
        );
      });
    }

    #[test]
    fn requester_does_not_have_enough_funds() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: CHARLIE_PARTIAL_FILLING_BUYS_5_TDFYS.saturating_mul(5),
              amount_to_send: CHARLIE_PARTIAL_FILLING_SELLS_100_TEMPS.saturating_mul(5),
            },],
          ),
          Error::<Test>::TraderHasNotEnoughTokenToSell
        );
      });
    }

    #[test]
    fn market_maker_swaps_buy_amount_is_greater_than_swap_sell_amount() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );
        let trade_request_mm_id =
          create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
            &context,
          );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm_id,
              amount_to_receive: BOB_SELLS_10_TDFYS.saturating_add(1),
              amount_to_send: BOB_BUYS_200_TEMPS,
            },],
          ),
          Error::<Test>::TraderCannotOversell
        );
      });
    }

    #[test]
    fn market_maker_swaps_buy_token_is_different_from_swap_sell_token() {
      new_test_ext().execute_with(|| {
        let context = Context::default()
          .set_oracle_status(true)
          .set_market_makers(vec![CHARLIE_ACCOUNT_ID])
          .mint_tdfy(ALICE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(CHARLIE_ACCOUNT_ID, ONE_TDFY)
          .mint_tdfy(BOB_ACCOUNT_ID, BOB_INITIAL_20_TDFYS)
          .create_temp_asset_and_metadata()
          .mint_temp(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS)
          .create_temp2_asset_metadata()
          .mint_temp2(CHARLIE_ACCOUNT_ID, CHARLIE_INITIAL_10000_TEMPS);

        let trade_request_id =
          create_bob_limit_swap_request_from_10_tdfys_to_200_temps_with_2_percents_slippage(
            &context,
          );

        create_charlie_limit_swap_request_from_4000_temps_to_200_tdfys_with_4_percents_slippage(
          &context,
        );

        // Add a new market maker swap with a different token TEMP2
        let trade_request_mm1_id = add_new_swap_and_assert_results(
          CHARLIE_ACCOUNT_ID,
          TEMP2_CURRENCY_ID,
          CHARLIE_SELLS_4000_TEMPS,
          CurrencyId::Tdfy,
          CHARLIE_BUYS_200_TDFYS,
          CURRENT_BLOCK_NUMBER,
          EXTRINSIC_HASH_1,
          true,
          SwapType::Limit,
          SLIPPAGE_4_PERCENTS,
        );

        assert_noop!(
          Oracle::confirm_swap(
            context.alice.clone(),
            trade_request_id,
            vec![SwapConfirmation {
              request_id: trade_request_mm1_id,
              amount_to_receive: BOB_SELLS_10_TDFYS,
              amount_to_send: BOB_BUYS_200_TEMPS,
            },],
          ),
          Error::<Test>::MarketMakerBuyTokenNotMatchSwapSellToken
        );
      });
    }
  }
}
