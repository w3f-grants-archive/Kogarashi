use crate::mock::{new_test_ext, ExampleFunction};
use crate::pallet::Config;
use crate::{self as nova_ivc};

use bn_254::{Fq, Fr};
use frame_support::dispatch::{DispatchErrorWithPostInfo, PostDispatchInfo};
use frame_support::{assert_ok, construct_runtime, parameter_types};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    DispatchError,
};
use zknova::{Bn254Driver, GrumpkinDriver};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        IvcPallet: nova_ivc::{Module, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
}

impl frame_system::Config for TestRuntime {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Index = u64;
    type Call = Call;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
}

impl Config for TestRuntime {
    type E1 = Bn254Driver;
    type E2 = GrumpkinDriver;
    type FC1 = ExampleFunction<Fr>;
    type FC2 = ExampleFunction<Fq>;
    type Event = Event;
}

#[cfg(test)]
mod ivc_pallet_tests {
    use super::*;
    use crate::FullcodecRng;
    use rand::SeedableRng;
    use zknova::Ivc;
    use zkstd::matrix::DenseVectors;

    fn get_rng() -> FullcodecRng {
        FullcodecRng::from_seed([
            0x59, 0x62, 0xbe, 0x5d, 0x76, 0x3d, 0x31, 0x8d, 0x17, 0xdb, 0x37, 0x32, 0x54, 0x06,
            0xbc, 0xe5,
        ])
    }

    #[test]
    fn trusted_setup() {
        new_test_ext().execute_with(|| {
            let rng = get_rng();
            assert_ok!(IvcPallet::trusted_setup(Origin::signed(1), rng));

            let rng = get_rng();
            assert_eq!(
                IvcPallet::trusted_setup(Origin::signed(1), rng),
                Err(DispatchErrorWithPostInfo {
                    post_info: PostDispatchInfo::from(()),
                    error: DispatchError::Other("already setup"),
                })
            );
        })
    }

    #[test]
    fn default_test() {
        let rng = get_rng();

        new_test_ext().execute_with(|| {
            assert_ok!(IvcPallet::trusted_setup(Origin::signed(1), rng));

            let pp = IvcPallet::public_params().unwrap();

            let z0_primary = DenseVectors::new(vec![Fr::from(0)]);
            let z0_secondary = DenseVectors::new(vec![Fq::from(0)]);
            let mut ivc = Ivc::<
                Bn254Driver,
                GrumpkinDriver,
                ExampleFunction<Fr>,
                ExampleFunction<Fq>,
            >::init(&pp, z0_primary, z0_secondary);

            for _ in 0..2 {
                let proof = ivc.prove_step(&pp);
                assert!(IvcPallet::verify(Origin::signed(1), proof).is_ok());
            }
        });
    }
}
