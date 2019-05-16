use beacon::primitives::H256;
use beacon::types::{BeaconState, BeaconBlock, UnsealedBeaconBlock};
use beacon::{Error as BeaconError, NoVerificationConfig, Inherent, Transaction};
use blockchain::traits::{Block as BlockT, BlockExecutor, BuilderExecutor, AsExternalities};
use lmd_ghost::JustifiableExecutor;
use parity_codec::{Encode, Decode};
use ssz::Digestible;

#[derive(Eq, PartialEq, Clone, Debug, Encode, Decode)]
pub struct Block(pub BeaconBlock);

impl BlockT for Block {
	type Identifier = H256;

	fn id(&self) -> H256 {
		H256::from_slice(
			Digestible::<sha2::Sha256>::hash(&self.0).as_slice()
		)
	}

	fn parent_id(&self) -> Option<H256> {
		if self.0.previous_block_root == H256::default() {
			None
		} else {
			Some(self.0.previous_block_root)
		}
	}
}

pub trait StateExternalities {
	fn state(&mut self) -> &mut BeaconState;
}

#[derive(Clone)]
pub struct State {
	state: BeaconState,
}

impl From<BeaconState> for State {
	fn from(state: BeaconState) -> Self {
		Self { state }
	}
}

impl Into<BeaconState> for State {
	fn into(self) -> BeaconState {
		self.state
	}
}

impl StateExternalities for State {
	fn state(&mut self) -> &mut BeaconState {
		&mut self.state
	}
}

impl AsExternalities<dyn StateExternalities> for State {
	fn as_externalities(&mut self) -> &mut (dyn StateExternalities + 'static) {
		self
	}
}

#[derive(Debug)]
pub enum Error {
	Beacon(BeaconError),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl std::error::Error for Error { }

impl From<BeaconError> for Error {
	fn from(error: BeaconError) -> Error {
		Error::Beacon(error)
	}
}

impl From<Error> for blockchain::chain::Error {
	fn from(error: Error) -> blockchain::chain::Error {
		blockchain::chain::Error::Executor(Box::new(error))
	}
}

pub struct Executor;

impl BlockExecutor for Executor {
	type Error = Error;
	type Block = Block;
	type Externalities = dyn StateExternalities + 'static;

	fn execute_block(
		&self,
		block: &Block,
		state: &mut Self::Externalities,
	) -> Result<(), Error> {
		let config = NoVerificationConfig::full();

		beacon::execute_block(&block.0, state.state(), &config)
			.map_err(|e| Error::Beacon(e))
	}
}

impl JustifiableExecutor for Executor {
	type ValidatorIndex = u64;

	fn justified_active_validators(
		&self,
		state: &mut Self::Externalities,
	) -> Result<Vec<Self::ValidatorIndex>, Self::Error> {
		let config = NoVerificationConfig::full();

		Ok(beacon::justified_active_validators(state.state(), &config))
	}

	fn justified_block_id(
		&self,
		state: &mut Self::Externalities,
	) -> Result<<Self::Block as BlockT>::Identifier, Self::Error> {
		let config = NoVerificationConfig::full();

		Ok(beacon::justified_root(state.state(), &config))
	}

	fn votes(
		&self,
		block: &Self::Block,
		state: &mut Self::Externalities,
	) -> Result<Vec<(Self::ValidatorIndex, <Self::Block as BlockT>::Identifier)>, Self::Error> {
		let config = NoVerificationConfig::full();

		Ok(beacon::block_vote_targets(&block.0, state.state(), &config)?)
	}
}

impl BuilderExecutor for Executor {
	type BuildBlock = UnsealedBeaconBlock;
	type Inherent = Inherent;
	type Extrinsic = Transaction;

	fn initialize_block(
		&self,
		parent_block: &Block,
		state: &mut Self::Externalities,
		inherent: Inherent,
	) -> Result<UnsealedBeaconBlock, Self::Error> {
		let config = NoVerificationConfig::full();

		beacon::initialize_block(&parent_block.0, state.state(), inherent, &config)
			.map_err(|e| Error::Beacon(e))
	}

	fn apply_extrinsic(
		&self,
		block: &mut UnsealedBeaconBlock,
		extrinsic: Transaction,
		state: &mut Self::Externalities,
	) -> Result<(), Self::Error> {
		let config = NoVerificationConfig::full();

		beacon::apply_transaction(block, state.state(), extrinsic, &config)
			.map_err(|e| Error::Beacon(e))
	}

	fn finalize_block(
		&self,
		block: &mut UnsealedBeaconBlock,
		state: &mut Self::Externalities,
	) -> Result<(), Self::Error> {
		let config = NoVerificationConfig::full();

		beacon::finalize_block(block, state.state(), &config)
			.map_err(|e| Error::Beacon(e))
	}
}