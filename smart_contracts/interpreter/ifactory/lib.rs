#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod ifactory {
    use ink_core::env::call::*;
    use ink_core::storage;

    const CONSTRUCTOR: [u8; 4] = [0x5E, 0xBD, 0x88, 0xD6];

    #[ink(storage)]
    struct Ifactory {
        idata_hash: storage::Value<Hash>,
        data_hash: storage::Value<Hash>,
    }

    struct NewIdata {
        instance: AccountId,
    }

    impl NewIdata {
        fn get_instance(&self) -> AccountId {
            self.instance
        }
    }

    impl Default for NewIdata {
        fn default() -> Self {
            Self {
                instance: AccountId::default(),
            }
        }
    }

    impl FromAccountId<EnvTypes> for NewIdata {
        fn from_account_id(new_instance: AccountId) -> Self {
            Self {
                instance: new_instance,
            }
        }
    }

    impl Ifactory {
        #[ink(constructor)]
        fn new(&mut self, idata_hash: Hash, data_hash: Hash) {
            self.idata_hash.set(idata_hash);
            self.data_hash.set(data_hash);
        }

        #[ink(message)]
        fn change_idata_hash(&mut self, idata_new_hash: Hash) {
            self.idata_hash.set(idata_new_hash);
        }

        #[ink(message)]
        fn change_data_hash(&mut self, data_new_hash: Hash) {
            self.data_hash.set(data_new_hash);
        }

        #[ink(message)]
        fn get_idata_hash(&self) -> Hash {
            *self.idata_hash
        }

        #[ink(message)]
        fn get_data_hash(&self) -> Hash {
            *self.data_hash
        }

        #[ink(message)]
        fn new_instance(&self) -> AccountId {
            let total_balance = self.env().balance();
            let selector = Selector::from(CONSTRUCTOR);
            InstantiateParams::<EnvTypes, NewIdata>::build(selector)
                .endowment(total_balance / 5)
                .using_code(*self.data_hash)
                .push_arg::<Hash>(&*self.idata_hash)
                .instantiate()
                .unwrap_or(NewIdata::default())
                .get_instance()
        }
    }
}
