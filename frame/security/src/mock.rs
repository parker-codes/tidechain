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

use crate::pallet as pallet_security;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_benchmarking::frame_support::traits::tokens::{DepositConsequence, WithdrawConsequence};
use frame_support::{
  parameter_types,
  traits::{
    fungible::{
      Inspect as FungibleInspect, Mutate as FungibleMutate, Transfer as FungibleTransfer,
    },
    fungibles::{Inspect, Mutate, Transfer},
    ConstU128, ConstU32,
  },
  PalletId,
};
use frame_system as system;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::{
  testing::Header,
  traits::{BlakeTwo256, IdentityLookup},
  DispatchError, DispatchResult, RuntimeDebug,
};
use std::marker::PhantomData;
use system::EnsureRoot;
use tidefi_primitives::CurrencyId;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;

#[derive(
  Encode,
  Decode,
  scale_info::TypeInfo,
  Default,
  Eq,
  PartialEq,
  Copy,
  Clone,
  RuntimeDebug,
  PartialOrd,
  Ord,
  MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Hash))]
pub struct AccountId(pub u64);

impl sp_std::fmt::Display for AccountId {
  fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<u64> for AccountId {
  fn from(account_id: u64) -> Self {
    Self(account_id)
  }
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
  pub enum Test where
    Block = Block,
    NodeBlock = Block,
    UncheckedExtrinsic = UncheckedExtrinsic,
  {
    System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
    Security: pallet_security::{Pallet, Call, Config, Storage, Event<T>},
    Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
    Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
  }
);

parameter_types! {
  pub const BlockHashCount: u64 = 250;
  pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
  type BaseCallFilter = frame_support::traits::Everything;
  type BlockWeights = ();
  type BlockLength = ();
  type DbWeight = ();
  type Origin = Origin;
  type Call = Call;
  type Index = u64;
  type BlockNumber = u64;
  type Hash = H256;
  type Hashing = BlakeTwo256;
  type AccountId = AccountId;
  type Lookup = IdentityLookup<Self::AccountId>;
  type Header = Header;
  type Event = Event;
  type BlockHashCount = BlockHashCount;
  type Version = ();
  type PalletInfo = PalletInfo;
  type AccountData = pallet_balances::AccountData<Balance>;
  type OnNewAccount = ();
  type OnKilledAccount = ();
  type SystemWeightInfo = ();
  type SS58Prefix = SS58Prefix;
  type OnSetCode = ();
  type MaxConsumers = ConstU32<16>;
}
pub const TDFY: Balance = 1_000_000_000_000;
parameter_types! {
  pub const ExistentialDeposit: Balance = TDFY;
  pub const MaxLocks: u32 = 50;
  pub const MaxReserves: u32 = 50;
}

parameter_types! {
  pub const AssetDeposit: u64 = 1;
  pub const ApprovalDeposit: u64 = 1;
  pub const StringLimit: u32 = 50;
  pub const MetadataDepositBase: u64 = 1;
  pub const MetadataDepositPerByte: u64 = 1;
}

impl pallet_assets::Config for Test {
  type Event = Event;
  type Balance = u128;
  type AssetId = u32;
  type Currency = Balances;
  type ForceOrigin = EnsureRoot<Self::AccountId>;
  type AssetDeposit = AssetDeposit;
  type MetadataDepositBase = MetadataDepositBase;
  type MetadataDepositPerByte = MetadataDepositPerByte;
  type ApprovalDeposit = ApprovalDeposit;
  type StringLimit = StringLimit;
  type Freezer = ();
  type Extra = ();
  type WeightInfo = ();
  type AssetAccountDeposit = ConstU128<0>;
}

impl pallet_balances::Config for Test {
  type Balance = Balance;
  type DustRemoval = ();
  type Event = Event;
  type ExistentialDeposit = ExistentialDeposit;
  type AccountStore = frame_system::Pallet<Test>;
  type MaxLocks = MaxLocks;
  type MaxReserves = MaxReserves;
  type ReserveIdentifier = [u8; 8];
  type WeightInfo = ();
}

impl pallet_security::Config for Test {
  type Event = Event;
  type WeightInfo = crate::weights::SubstrateWeight<Test>;
}

parameter_types! {
  pub const TidefiPalletId: PalletId = PalletId(*b"wrpr*pal");
}

// this is only the mock for benchmarking, it's implemented directly in the runtime
pub struct Adapter<AccountId> {
  phantom: PhantomData<AccountId>,
}

impl Inspect<AccountId> for Adapter<AccountId> {
  type AssetId = CurrencyId;
  type Balance = Balance;

  fn total_issuance(asset: Self::AssetId) -> Self::Balance {
    match asset {
      CurrencyId::Tdfy => Balances::total_issuance(),
      CurrencyId::Wrapped(asset_id) => Assets::total_issuance(asset_id),
    }
  }

  fn balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
    match asset {
      CurrencyId::Tdfy => Balances::balance(who),
      CurrencyId::Wrapped(asset_id) => Assets::balance(asset_id, who),
    }
  }

  fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
    match asset {
      CurrencyId::Tdfy => Balances::minimum_balance(),
      CurrencyId::Wrapped(asset_id) => Assets::minimum_balance(asset_id),
    }
  }

  fn reducible_balance(asset: Self::AssetId, who: &AccountId, keep_alive: bool) -> Self::Balance {
    match asset {
      CurrencyId::Tdfy => Balances::reducible_balance(who, keep_alive),
      CurrencyId::Wrapped(asset_id) => Assets::reducible_balance(asset_id, who, keep_alive),
    }
  }

  fn can_deposit(
    asset: Self::AssetId,
    who: &AccountId,
    amount: Self::Balance,
    mint: bool,
  ) -> DepositConsequence {
    match asset {
      CurrencyId::Tdfy => Balances::can_deposit(who, amount, mint),
      CurrencyId::Wrapped(asset_id) => Assets::can_deposit(asset_id, who, amount, mint),
    }
  }

  fn can_withdraw(
    asset: Self::AssetId,
    who: &AccountId,
    amount: Self::Balance,
  ) -> WithdrawConsequence<Self::Balance> {
    match asset {
      CurrencyId::Tdfy => Balances::can_withdraw(who, amount),
      CurrencyId::Wrapped(asset_id) => Assets::can_withdraw(asset_id, who, amount),
    }
  }
}

impl Mutate<AccountId> for Adapter<AccountId> {
  fn mint_into(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> DispatchResult {
    match asset {
      CurrencyId::Tdfy => Balances::mint_into(who, amount),
      CurrencyId::Wrapped(asset_id) => Assets::mint_into(asset_id, who, amount),
    }
  }

  fn burn_from(
    asset: Self::AssetId,
    who: &AccountId,
    amount: Balance,
  ) -> Result<Balance, DispatchError> {
    match asset {
      CurrencyId::Tdfy => Balances::burn_from(who, amount),
      CurrencyId::Wrapped(asset_id) => Assets::burn_from(asset_id, who, amount),
    }
  }
}

impl Transfer<AccountId> for Adapter<AccountId>
where
  Assets: Transfer<AccountId>,
{
  fn transfer(
    asset: Self::AssetId,
    source: &AccountId,
    dest: &AccountId,
    amount: Self::Balance,
    keep_alive: bool,
  ) -> Result<Balance, DispatchError> {
    match asset {
      CurrencyId::Tdfy => {
        <Balances as FungibleTransfer<AccountId>>::transfer(source, dest, amount, keep_alive)
      }
      CurrencyId::Wrapped(asset_id) => {
        <Assets as Transfer<AccountId>>::transfer(asset_id, source, dest, amount, keep_alive)
      }
    }
  }
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
  let _alice = 1u64;
  let mut t = system::GenesisConfig::default()
    .build_storage::<Test>()
    .unwrap();
  pallet_balances::GenesisConfig::<Test>::default()
    .assimilate_storage(&mut t)
    .unwrap();

  t.into()
}
